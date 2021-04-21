import { Construct, NestedStack } from "@aws-cdk/core";
import { IBucket } from '@aws-cdk/aws-s3';
import { RestApi, LambdaIntegration, Cors } from '@aws-cdk/aws-apigateway';
import { GfaFunction } from './function/gfa-function';

export interface StopsStackProps {
    stopsBucket: IBucket,
    stopsPath: string,
    api: RestApi
}

export class StopsStack extends NestedStack {
    constructor(scope: Construct, id: string, props: StopsStackProps) {
        super(scope, id);

        const getStops = new GfaFunction(this, 'get-stops', {
            name: 'get-stops',
            environment: {
                STOPS_BUCKET: props.stopsBucket.bucketName,
                STOPS_PATH: props.stopsPath,
            }
        });
        props.stopsBucket.grantRead(getStops.handler, props.stopsPath);

        const getStopsIntegration = new LambdaIntegration(getStops.handler, {
            proxy: true,
        });

        const resource = props.api.root.addResource('stops');
        resource.addCorsPreflight({
            allowOrigins: Cors.ALL_ORIGINS,
            allowMethods: ['GET'],
            allowHeaders: ['Content-Type', 'Accept']
        });
        resource.addMethod('GET', getStopsIntegration);
    }
}
