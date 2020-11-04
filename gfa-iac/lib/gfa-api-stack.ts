import { NestedStack, NestedStackProps } from '@aws-cdk/aws-cloudformation';
import { IBucket } from '@aws-cdk/aws-s3';
import { Construct } from '@aws-cdk/core';
import { Cors, LambdaIntegration, LambdaRestApi, PassthroughBehavior, RestApi } from '@aws-cdk/aws-apigateway';
import { functionCreator } from './function-creator';

interface ApiStackProps extends NestedStackProps {
    version: string,
    artifactsBucket: IBucket,
    stopsBucket: IBucket,
    stopsPath: string,
    webOrigin: string,
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
        const resource = this.api.root.addResource('stops');
        resource.addMethod('GET', new LambdaIntegration(getStops, {
            proxy: false,
            allowTestInvoke: true,
            passthroughBehavior: PassthroughBehavior.NEVER,
            integrationResponses: [
                {
                    statusCode: '200',
                    responseTemplates: {
                        'application/json': JSON.stringify({ state: 'ok', stops: '$util.escapeJavaScript($input.body)' }),
                    },
                    responseParameters: {
                        'method.response.header.Content-Type': "'application/json'",
                        'method.response.header.Access-Control-Allow-Origin': "'*'",
                        'method.response.header.Access-Control-Allow-Methods': "'GET'",
                        'method.response.header.Access-Control-Allow-Headers': "'Content-Type, Accept'",
                    },
                },
                {
                    selectionPattern: '(\n|.)+',
                    statusCode: "400",
                    responseTemplates: {
                        'application/json': JSON.stringify({ state: 'error', message: "$util.escapeJavaScript($input.path('$.errorMessage'))" })
                    },
                    responseParameters: {
                        'method.response.header.Content-Type': "'application/json'",
                        'method.response.header.Access-Control-Allow-Origin': "'*'",
                        'method.response.header.Access-Control-Allow-Methods': "'GET'",
                        'method.response.header.Access-Control-Allow-Headers': "'Content-Type, Accept'",
                    },
                },
            ],
        }));
    }
}
