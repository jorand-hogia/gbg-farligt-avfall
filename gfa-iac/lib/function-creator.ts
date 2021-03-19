import { Construct, Duration } from "@aws-cdk/core";
import { IBucket } from '@aws-cdk/aws-s3';
import { LambdaInvoke } from '@aws-cdk/aws-stepfunctions-tasks';
import { Function, Runtime, S3Code } from '@aws-cdk/aws-lambda';

interface FunctionProps {
    timeout?: Duration,
    environment?: { [key: string]: string },
}
interface FunctionWithInvokeTaskProps extends FunctionProps {
    outputPath?: string,
}
export const functionWithInvokeCreator = (artifactsBucket: IBucket, version: string) => {
    return (scope: Construct, name: string, props: FunctionWithInvokeTaskProps = {}): [Function, LambdaInvoke] => {
        const fn = createFunction(artifactsBucket, version, scope, name, props);
        const invokeTask = new LambdaInvoke(scope, `invoke-${name}`, {
            lambdaFunction: fn,
            outputPath: props.outputPath,
        });
        return [fn, invokeTask];
    }
}

export const functionCreator = (artifactsBucket: IBucket, version: string) => {
    return (scope: Construct, name: string, props: FunctionProps = {}): Function => {
        return createFunction(artifactsBucket, version, scope, name, props);
    }
}

const createFunction = (artifactsBucket: IBucket, version: string, scope: Construct, name: string, props: FunctionProps = {}): Function => {
    return new Function(scope, `gfa-${name}`, {
        code: new S3Code(artifactsBucket, `gfa-${name}-${version}`),
        runtime: Runtime.PROVIDED,
        handler: 'doesnt.matter',
        timeout: props.timeout || Duration.seconds(10),
        environment: props.environment || {},
    })
}