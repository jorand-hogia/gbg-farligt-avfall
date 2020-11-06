import { NestedStack, NestedStackProps } from '@aws-cdk/aws-cloudformation';
import { IBucket } from '@aws-cdk/aws-s3';
import { Construct } from '@aws-cdk/core';
import { Cors, LambdaIntegration, LambdaRestApi, PassthroughBehavior, RestApi, JsonSchemaType, JsonSchemaVersion } from '@aws-cdk/aws-apigateway';
import { functionCreator } from './function-creator';

interface ApiStackProps extends NestedStackProps {
    version: string,
    artifactsBucket: IBucket,
    stopsBucket: IBucket,
    stopsPath: string,
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
    }
}
