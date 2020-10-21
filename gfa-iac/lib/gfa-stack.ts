import { Code, Function, Runtime, CfnParametersCode } from '@aws-cdk/aws-lambda';
import { LambdaInvoke } from '@aws-cdk/aws-stepfunctions-tasks';
import { Parallel, StateMachine } from '@aws-cdk/aws-stepfunctions';
import { App, Duration, PhysicalName, Stack, StackProps } from '@aws-cdk/core';
import { Table, AttributeType, BillingMode } from '@aws-cdk/aws-dynamodb';
import { Secret } from "@aws-cdk/aws-secretsmanager";

export class GbgFarligtAvfallStack extends Stack {
  public readonly scraperCode: CfnParametersCode;
  public readonly saveEventsCode: CfnParametersCode;
  public readonly preProcessStopsCode: CfnParametersCode;
  public readonly saveStopsCode: CfnParametersCode;
      
  constructor(app: App, id: string, props?: StackProps) {
    super(app, id, props);
    this.scraperCode = Code.fromCfnParameters();
    this.saveEventsCode = Code.fromCfnParameters();
    this.preProcessStopsCode = Code.fromCfnParameters();
    this.saveStopsCode = Code.fromCfnParameters();

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
      lambdaFunction: scraper,
      outputPath: '$.Payload'
    });

    const saveEvents = new Function(this, 'gfa-save-events', {
      code: this.saveEventsCode,
      handler: 'doesnt.matter',
      runtime: Runtime.PROVIDED,
      timeout: Duration.seconds(10)
    });
    const saveEventsTask = new LambdaInvoke(this, 'gfa-task-save-events', {
      lambdaFunction: saveEvents
    });

    const preProcessStops = new Function(this, 'gfa-pre-process-stops', {
      code: this.preProcessStopsCode,
      handler: 'doesnt.matter',
      runtime: Runtime.PROVIDED,
      timeout: Duration.seconds(10),
      environment: {
        GEOCODING_API_KEY: `${
          Secret.fromSecretName(this, 'geocoding-api-key', 'mapquest-api-key').secretValue
        }`
      }
    });
    const preProcessStopsTask = new LambdaInvoke(this, 'gfa-task-pre-process-stops', {
      lambdaFunction: preProcessStops
    });

    const saveStops = new Function(this, 'gfa-save-stops', {
      code: this.saveStopsCode,
      handler: 'doesnt.matter',
      runtime: Runtime.PROVIDED,
      timeout: Duration.seconds(10),
    });
    const saveStopsTask = new LambdaInvoke(this, 'gfa-task-save-stops', {
      lambdaFunction: saveStops,
    });

    const scrapeAndSaveFlow = new StateMachine(this, 'gfa-scrape-and-save', {
      definition: scrapeTask
        .next(new Parallel(this, 'process-scrape-results', {})
          .branch(saveEventsTask)
          .branch(preProcessStopsTask)
            .next(saveStopsTask)
        ),
      timeout: Duration.minutes(5)
    });

  }
}
