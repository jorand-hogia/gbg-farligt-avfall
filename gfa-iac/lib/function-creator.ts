import { Construct, Duration } from "@aws-cdk/core";
import { IBucket } from '@aws-cdk/aws-s3';
import { LambdaInvoke } from '@aws-cdk/aws-stepfunctions-tasks';
import { Function, Runtime, S3Code } from '@aws-cdk/aws-lambda';

interface FunctionWithInvokeTaskProps {
    timeout?: Duration,
    environment?: { [key: string]: string },
    outputPath?: string,
}
export const functionCreator = (artifactsBucket: IBucket, version: string) => {
    return (scope: Construct, name: string, props: FunctionWithInvokeTaskProps = {}): [Function, LambdaInvoke] => {
        const fn = new Function(scope, `gfa-${name}`, {
            code: new S3Code(artifactsBucket, `gfa-${name}-${version}`),
            runtime: Runtime.PROVIDED,
            handler: 'doesnt.matter',
            timeout: props.timeout || Duration.seconds(10),
            environment: props.environment || {},
        });
        const invokeTask = new LambdaInvoke(scope, `invoke-${name}`, {
            lambdaFunction: fn,
            outputPath: props.outputPath,
        });
        return [fn, invokeTask];
    }
}
