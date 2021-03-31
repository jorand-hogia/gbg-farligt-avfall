import { Construct, Duration } from "@aws-cdk/core";
import { Bucket } from '@aws-cdk/aws-s3';
import { Function, Runtime, S3Code } from '@aws-cdk/aws-lambda';

export interface GfaFunctionProps {
    name: string,
    timeout?: Duration,
    environment?: { [key: string]: string },
}

export class GfaFunction extends Construct {

    public readonly handler: Function;

    constructor(scope: Construct, id: string, props: GfaFunctionProps) {
        super(scope, id);

        const artifactsBucketName = scope.node.tryGetContext('artifactsBucketName');
        const artifactsBucket = Bucket.fromBucketName(this, 'artifactsBucket', artifactsBucketName);
        const version = scope.node.tryGetContext('version');

        this.handler = new Function(scope, `gfa-${props.name}`, {
            code: new S3Code(artifactsBucket, `gfa-${props.name}-${version}`),
            runtime: Runtime.PROVIDED,
            handler: 'doesnt.matter',
            timeout: props.timeout || Duration.seconds(10),
            environment: props.environment || {},
        })
    }
}