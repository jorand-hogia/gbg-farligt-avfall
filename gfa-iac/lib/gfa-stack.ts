import { Code, Function, Runtime, CfnParametersCode } from '@aws-cdk/aws-lambda';
import { LambdaInvoke } from '@aws-cdk/aws-stepfunctions-tasks';
import { StateMachine } from '@aws-cdk/aws-stepfunctions';
import { App, Duration, Stack, StackProps } from '@aws-cdk/core';
import { Table, AttributeType, BillingMode } from '@aws-cdk/aws-dynamodb';

export class GbgFarligtAvfallStack extends Stack {
  public readonly scraperCode: CfnParametersCode;
  public readonly eventsCode: CfnParametersCode;
      
  constructor(app: App, id: string, props?: StackProps) {
    super(app, id, props);
    this.scraperCode = Code.fromCfnParameters();
    this.eventsCode = Code.fromCfnParameters();

    const eventsDb = new Table(this, 'gfa-events-db', {
      partitionKey: { name: 'event-date', type: AttributeType.STRING },
      sortKey: { name: 'location-id', type: AttributeType.STRING },
      billingMode: BillingMode.PAY_PER_REQUEST,
    });

    const scraper = new Function(this, 'gfa-scraper', {
      code: this.scraperCode,
      handler: 'doesnt.matter',
      runtime: Runtime.PROVIDED,
      timeout: Duration.seconds(10),
    });
    const scrapeTask = new LambdaInvoke(this, 'gfa-task-scrape', {
      lambdaFunction: scraper
    });

    const saveEvents = new Function(this, 'gfa-save-events', {
      code: this. eventsCode,
      handler: 'doesnt.matter',
      runtime: Runtime.PROVIDED,
      timeout: Duration.seconds(10)
    });
    const saveEventsTask = new LambdaInvoke(this, 'gfa-task-save-events', {
      lambdaFunction: saveEvents
    });

    const scrapeAndSaveFlow = new StateMachine(this, 'gfa-scrape-and-save', {
      definition: scrapeTask
        .next(saveEventsTask),
      timeout: Duration.minutes(5)
    });

  }
}
