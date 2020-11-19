import { NestedStack, NestedStackProps } from '@aws-cdk/aws-cloudformation';
import { Construct } from '@aws-cdk/core';
import { Cors, LambdaIntegration, RestApi, SecurityPolicy, Stage } from '@aws-cdk/aws-apigateway';
import { IFunction } from '@aws-cdk/aws-lambda';
import { Certificate } from '@aws-cdk/aws-certificatemanager';
import { CertificateValidation } from '@aws-cdk/aws-certificatemanager';
import { ARecord, HostedZone, RecordTarget } from '@aws-cdk/aws-route53';
import * as targets from '@aws-cdk/aws-route53-targets';

export interface LambdaEndpoint {
    lambda: IFunction,
    resource: string,
    methods: string[],
}
interface ApiStackProps extends NestedStackProps {
    hostedZoneId?: string,
    domainName?: string,
    lambdaEndpoints: LambdaEndpoint[],
}

export class ApiStack extends NestedStack {

    public readonly apiUrl: string;

    constructor(scope: Construct, id: string, props: ApiStackProps) {
        super(scope, id, props);

        const api = new RestApi(this, 'gfa-api');
        this.apiUrl = api.url;

        props.lambdaEndpoints.forEach(endpoint => {
            const integration = new LambdaIntegration(endpoint.lambda, {
                proxy: true,
            });
            const resource = api.root.getResource(endpoint.resource)
                || api.root.addResource(endpoint.resource);
            resource.addCorsPreflight({
                allowOrigins: Cors.ALL_ORIGINS,
                allowMethods: endpoint.methods,
                allowHeaders: ['Content-Type', 'Accept']
            })
            endpoint.methods.forEach(method => {
                resource.addMethod(method, integration);
            })
        })

        if (props.hostedZoneId && props.domainName) {
            const apiDomainName = `gfa-api.${props.domainName}`;
            const hostedZone = HostedZone.fromHostedZoneAttributes(this, 'e-hostedzone', {
                hostedZoneId: props.hostedZoneId,
                zoneName: props.domainName,
            });
            const apiCert = new Certificate(this, 'gfa-api-certificate', {
                domainName: apiDomainName,
                validation: CertificateValidation.fromDns(hostedZone),
            });
            api.addDomainName('gfa-api-domain', {
                domainName: apiDomainName,
                certificate: apiCert, 
                securityPolicy: SecurityPolicy.TLS_1_2,
            });
            new ARecord(this, 'gfa-api-domain-record', {
                zone: hostedZone,
                target: RecordTarget.fromAlias(new targets.ApiGateway(api)),
                recordName: apiDomainName,
            });
            this.apiUrl = `https://${apiDomainName}`;
        }
    }
}
