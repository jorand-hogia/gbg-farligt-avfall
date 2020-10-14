import * as lambda from '@aws-cdk/aws-lambda';
import * as dynamodb from '@aws-cdk/aws-dynamodb';
import { App, Duration, Stack, StackProps } from '@aws-cdk/core';
import { BillingMode } from '@aws-cdk/aws-dynamodb';

export class GbgFarligtAvfallStack extends Stack {
  public readonly scraperCode: lambda.CfnParametersCode;
  public readonly eventsCode: lambda.CfnParametersCode;
      
  constructor(app: App, id: string, props?: StackProps) {
    super(app, id, props);
    this.scraperCode = lambda.Code.fromCfnParameters();
    this.eventsCode = lambda.Code.fromCfnParameters();

    const gfaEventsDb = new dynamodb.Table(this, 'gfa-events-db', {
      partitionKey: { name: 'event-date', type: dynamodb.AttributeType.STRING },
      sortKey: { name: 'location-id', type: dynamodb.AttributeType.STRING },
      billingMode: BillingMode.PAY_PER_REQUEST,
    });

    const gfaScraper = new lambda.Function(this, 'gfa-scraper', {
      code: this.scraperCode,
      handler: 'doesnt.matter',
      runtime: lambda.Runtime.PROVIDED,
      timeout: Duration.seconds(10),
    });
    const gfaEvents = new lambda.Function(this, 'gfa-events', {
      code: this. eventsCode,
      handler: 'doesnt.matter',
      runtime: lambda.Runtime.PROVIDED,
      timeout: Duration.seconds(10)
    });

  }
}
