import * as codedeploy from '@aws-cdk/aws-codedeploy';
import * as lambda from '@aws-cdk/aws-lambda';
import { App, Stack, StackProps } from '@aws-cdk/core';
      
export class GbgFarligtAvfallStack extends Stack {
  public readonly lambdaCode: lambda.CfnParametersCode;
      
  constructor(app: App, id: string, props?: StackProps) {
    super(app, id, props);
      
    this.lambdaCode = lambda.Code.fromCfnParameters();
      
    const func = new lambda.Function(this, 'gbg-farligt-avfall-poller', {
      code: this.lambdaCode,
      handler: 'doesnt.matter',
      runtime: lambda.Runtime.PROVIDED,
    });
  }
}
