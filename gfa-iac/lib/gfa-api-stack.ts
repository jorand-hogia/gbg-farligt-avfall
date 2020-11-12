import { NestedStack, NestedStackProps } from '@aws-cdk/aws-cloudformation';
import { IBucket } from '@aws-cdk/aws-s3';
import { Construct } from '@aws-cdk/core';
import { Cors, LambdaIntegration, LambdaRestApi, RestApi } from '@aws-cdk/aws-apigateway';
import { functionCreator } from './function-creator';
import { Certificate } from '@aws-cdk/aws-certificatemanager';
import { CertificateValidation } from '@aws-cdk/aws-certificatemanager';
import { ARecord, HostedZone, RecordTarget } from '@aws-cdk/aws-route53';
import * as targets from '@aws-cdk/aws-route53-targets';

interface ApiStackProps extends NestedStackProps {
    version: string,
    artifactsBucket: IBucket,
    stopsBucket: IBucket,
    stopsPath: string,
    hostedZoneId?: string,
    apiDomainName?: string,
}

export class ApiStack extends NestedStack {

    public readonly api: LambdaRestApi;

    constructor(scope: Construct, id: string, props: ApiStackProps) {
        super(scope, id, props);

        const newFunction = functionCreator(props.artifactsBucket, props.version);
        const getStops = newFunction(this, 'get-stops', {
            environment: {
                STOPS_BUCKET: props.stopsBucket.bucketName,
                STOPS_PATH: props.stopsPath,
            }
        });
        props.stopsBucket.grantRead(getStops, props.stopsPath);

        this.api = new RestApi(this, 'gfa-api', {
            defaultCorsPreflightOptions: {
                allowOrigins: Cors.ALL_ORIGINS,
                allowMethods: ['GET'],
                allowHeaders: ['Content-Type', 'Accept'],
            },
        });
        const getStopsIntegration = new LambdaIntegration(getStops, {
            proxy: true,
        });
        const stopsResource = this.api.root.addResource('stops');
        stopsResource.addMethod('GET', getStopsIntegration);

        if (props.hostedZoneId && props.apiDomainName) {
            const hostedZone = HostedZone.fromHostedZoneId(this, 'external-hostedzone', props.hostedZoneId);
            const apiCert = new Certificate(this, 'gfa-api-certificate', {
                domainName: props.apiDomainName,
                validation: CertificateValidation.fromDns(hostedZone),
            });
            this.api.addDomainName('gfa-api-domain', {
                domainName: props.apiDomainName,
                certificate: apiCert, 
            });
            new ARecord(this, 'gfa-api-domain-record', {
                zone: hostedZone,
                target: RecordTarget.fromAlias(new targets.ApiGateway(this.api)),
                recordName: props.apiDomainName,
            });
        }
    }
}
