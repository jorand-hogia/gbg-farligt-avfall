import * as lambda from '@aws-cdk/aws-lambda';
import * as dynamodb from '@aws-cdk/aws-dynamodb';
import { App, Duration, Stack, StackProps } from '@aws-cdk/core';
import { BillingMode } from '@aws-cdk/aws-dynamodb';

export class GbgFarligtAvfallStack extends Stack {
  public readonly lambdaCode: lambda.CfnParametersCode;
      
  constructor(app: App, id: string, props?: StackProps) {
    super(app, id, props);
    this.lambdaCode = lambda.Code.fromCfnParameters();

    const gfaEvents = new dynamodb.Table(this, 'gfa-events', {
      partitionKey: { name: 'event-date', type: dynamodb.AttributeType.STRING },
      sortKey: { name: 'district-and-street', type: dynamodb.AttributeType.STRING },
      billingMode: BillingMode.PAY_PER_REQUEST,
    });

    const gfaPoller = new lambda.Function(this, 'gfa-poller', {
      code: this.lambdaCode,
      handler: 'doesnt.matter',
      runtime: lambda.Runtime.PROVIDED,
      timeout: Duration.seconds(10),
      environment: {
        EVENTS_TABLE: gfaEvents.tableName,
      },
    });
    gfaEvents.grantWriteData(gfaPoller);

  }
}
