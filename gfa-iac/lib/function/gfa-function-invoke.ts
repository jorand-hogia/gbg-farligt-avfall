import { Construct } from "@aws-cdk/core";
import { Function } from '@aws-cdk/aws-lambda';
import { LambdaInvoke } from '@aws-cdk/aws-stepfunctions-tasks';
import { GfaFunction, GfaFunctionProps } from "./gfa-function";

export interface GfaFunctionWithInvokeTaskProps extends GfaFunctionProps {
    outputPath?: string
}

export class GfaFunctionWithInvokeTask extends Construct {

    public readonly handler: Function;
    public readonly task: LambdaInvoke;

    constructor(scope: Construct, id: string, props: GfaFunctionWithInvokeTaskProps) {
        super(scope, id);

        this.handler = new GfaFunction(this, `fn-${props.name}`, props).handler;
        this.task = new LambdaInvoke(this, `invoke-${props.name}`, {
            lambdaFunction: this.handler,
            outputPath: props.outputPath,
        });
    }
}
