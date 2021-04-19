const https = require('https');
const url = require('url');
const AWS = require('aws-sdk')

const baseUrl = 'https://api.sendgrid.com/v3';

exports.handler = async function (event, context) {
    const { ResourceProperties: properties, OldResourceProperties: oldProperties, RequestType: requestType, PhysicalResourceId: physicalId } = event;
    switch (requestType) {
        case 'Create':
            return handleCreate(properties.apiKey, properties.domain, properties.hostedZoneId);
        case 'Update':
            return handleUpdate(physicalId, properties, oldProperties);
        case 'Delete':
            return handleDelete(properties.apiKey, properties.hostedZoneId, physicalId);
    }
}

const handleCreate = async (apiKey, domain, hostedZoneId) => {
    const authenticationResponse = await createDomainAuthentication(apiKey, domain)
        .catch(error => {
            throw new Error('Failed to authenticate domain: ' + error);
        })
    const {id, dns} = authenticationResponse;
    await modifyAuthenticationRecords(hostedZoneId, dns, 'CREATE')
        .catch(error => {
            // Delete authentication in SendGrid?
            throw new Error('Failed to add CNAME records to Route53: ' + error);
        });
    await retry(() => validateDomainAuthentication(apiKey, id), 'Failed to validate domain authentication')
        .catch(error => {
            throw new Error('Failed to validate domain authentication: ' + error);
        });
    return successfulCloudFormationResponse(id.toString(10), {});
}

const handleUpdate = (id, properties, oldProperties) => {
    const domainChanged = properties.domain !== oldProperties.domain;
    const hostedZoneChanged = properties.hostedZoneId !== oldProperties.hostedZoneId;
    const apiKeyChanged = properties.apiKey !== oldProperties.apiKey;

    if (!domainChanged && !hostedZoneChanged && apiKeyChanged) {
        return successfulCloudFormationResponse(id, {});
    }
    return handleCreate(properties.apiKey, properties.domain, properties.hostedZoneId);
}

const handleDelete = async (apiKey, hostedZoneId, id) => {
    const { dns } = await getDomainAuthentication(apiKey, id)
        .catch(error => {
            throw new Error(`Failed to get domain authentication with id ${id}: ` + error);
        });
    await modifyAuthenticationRecords(hostedZoneId, dns, 'DELETE')
        .catch(error => {
            throw new Error('Failed to delete CNAME records from Route53: ' + error);
        });
    await deleteDomainAuthentication(apiKey, id)
        .catch(error => {
            throw new Error('Failed to delete domain authentication: ' + error);
        })
    return successfulCloudFormationResponse(id, {});
}

const successfulCloudFormationResponse = (physicalResourceId, responseData) => {
    return {
        PhysicalResourceId: physicalResourceId,
        Data: responseData
    };
}

const modifyAuthenticationRecords = (hostedZoneId, resourcesToModify, action) => {
    const route53 = new AWS.Route53();
    return route53.changeResourceRecordSets({
        HostedZoneId: hostedZoneId,
        ChangeBatch: {
            Changes: Object.values(resourcesToModify).map(record => ({
                Action: action,
                ResourceRecordSet: {
                    Name: record.host,
                    Type: record.type.toUpperCase(),
                    TTL: 300,
                    ResourceRecords: [{
                        Value: record.data
                    }]
                }
            }))
        }
    }).promise();
}

const createDomainAuthentication = (apiKey, domain) => {
    const uri = new url.URL(baseUrl + '/whitelabel/domains');
    return makeRequest(uri, {
        method: 'POST',
        headers: {
            Authorization: 'Bearer ' + apiKey
        }
    }, JSON.stringify({
        domain,
    })).then(response => JSON.parse(response));
}

const validateDomainAuthentication = (apiKey, id) => {
    const uri = new url.URL(baseUrl + `/whitelabel/domains/${id}/validate`);
    return makeRequest(uri, {
        method: 'POST',
        headers: {
            Authorization: 'Bearer ' + apiKey
        }
    }).then(response => {
        response = JSON.parse(response)
        if (!response.valid) {
            throw new Error('Invalid domain authentication');
        }
        return response;
    });
}

const getDomainAuthentication = (apiKey, id) => {
    const uri = new url.URL(baseUrl + `/whitelabel/domains/${id}`);
    return makeRequest(uri, {
        method: 'GET',
        headers: {
            Authorization: 'Bearer ' + apiKey
        }
    }).then(response => JSON.parse(response));
}

const deleteDomainAuthentication = (apiKey, id) => {
    const uri = new url.URL(baseUrl + `/whitelabel/domains/${id}`);
    return makeRequest(uri, {
        method: 'DELETE',
        headers: {
            Authorization: 'Bearer ' + apiKey
        }
    });
}

const retry = async (fn, errorMessage, retryCount = 0, lastError = null) => {
    try {
        return await fn();
    } catch (error) {
        if (retryCount > 15) {
            throw new Error(lastError);
        }
        if (errorMessage) {
            console.warn(errorMessage);
        }
        const delayInMs = 2 ** retryCount * 10;
        await delay(delayInMs);
        return retry(fn, errorMessage, retryCount + 1, error);
    }
}

const delay = ms =>
  new Promise(resolve => setTimeout(resolve, ms));

const makeRequest = (uri, options, data) => new Promise((resolve, reject) => {
    if (!uri instanceof url.URL) {
        throw new Error('url must be an instance of url.URL');
    }
    const fullPath = uri.pathname.concat(uri.search ? uri.search : '');
    const req = https.request({
        hostname: uri.hostname,
        path: fullPath,
        method: options.method,
        headers: options.headers,
        protocol: uri.protocol
    }, (res) => {
        res.setEncoding('utf8');
        if (res.statusCode < 200 || res.statusCode > 299) {
            res.on('data', chunk => {});
            return reject(new Error("Bad status code: " + res.statusCode));
        }
        let responseBody = '';
        res.on('data', chunk => {
            responseBody += chunk;
        });
        res.on('end', () => {
            return resolve(responseBody);
        });
    });
    req.on('error', err => {
        return reject(err);
    });
    if (data) {
        req.write(data);
    }
    req.end();
});
