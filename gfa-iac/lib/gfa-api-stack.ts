import { NestedStack } from '@aws-cdk/aws-cloudformation';
import { Construct } from '@aws-cdk/core';
import { RestApi, SecurityPolicy } from '@aws-cdk/aws-apigateway';
import { Certificate } from '@aws-cdk/aws-certificatemanager';
import { CertificateValidation } from '@aws-cdk/aws-certificatemanager';
import { ARecord, HostedZone, RecordTarget } from '@aws-cdk/aws-route53';
import * as targets from '@aws-cdk/aws-route53-targets';

export class ApiStack extends NestedStack {

    public readonly api: RestApi;
    public readonly externalUrl?: string;

    constructor(scope: Construct, id: string) {
        super(scope, id);

        this.api = new RestApi(this, 'gfa-api');

        const hostedZoneId = scope.node.tryGetContext('hostedZoneId');
        const domainName = scope.node.tryGetContext('domainName');

        if (hostedZoneId && domainName) {
            const apiDomainName = `gfa-api.${domainName}`;
            const hostedZone = HostedZone.fromHostedZoneAttributes(this, 'e-hostedzone', {
                hostedZoneId: hostedZoneId,
                zoneName: domainName,
            });
            const apiCert = new Certificate(this, 'gfa-api-certificate', {
                domainName: apiDomainName,
                validation: CertificateValidation.fromDns(hostedZone),
            });
            this.api.addDomainName('gfa-api-domain', {
                domainName: apiDomainName,
                certificate: apiCert, 
                securityPolicy: SecurityPolicy.TLS_1_2,
            });
            new ARecord(this, 'gfa-api-domain-record', {
                zone: hostedZone,
                target: RecordTarget.fromAlias(new targets.ApiGateway(this.api)),
                recordName: apiDomainName,
            });
            this.externalUrl = `https://${apiDomainName}`;
        }
    }
}
