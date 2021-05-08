import { Construct, Duration, RemovalPolicy } from '@aws-cdk/core';
import { NestedStack, NestedStackProps } from '@aws-cdk/aws-cloudformation';
import { StateMachine, TaskInput } from '@aws-cdk/aws-stepfunctions';
import { SnsPublish } from '@aws-cdk/aws-stepfunctions-tasks';
import { Table, AttributeType, BillingMode, ProjectionType } from '@aws-cdk/aws-dynamodb';
import { ITopic } from '@aws-cdk/aws-sns';
import { Rule, Schedule } from '@aws-cdk/aws-events';
import { SfnStateMachine } from '@aws-cdk/aws-events-targets';
import { GfaFunctionWithInvokeTask } from './function/gfa-function-invoke';

export interface EventsIngestionStackProps extends NestedStackProps {
  alertTopic: ITopic,
}

export class EventsIngestionStack extends NestedStack {

  public readonly eventsTable: Table;
  public readonly locationIndex: string = 'byLocationId'; 

  constructor(scope: Construct, id: string, props: EventsIngestionStackProps) {
    super(scope, id, props);

    this.eventsTable = new Table(this, 'events-db', {
      partitionKey: { name: 'event_date', type: AttributeType.STRING },
      sortKey: { name: 'location_id', type: AttributeType.STRING },
      billingMode: BillingMode.PAY_PER_REQUEST,
      removalPolicy: RemovalPolicy.DESTROY
    });
    this.eventsTable.addGlobalSecondaryIndex({
      indexName: this.locationIndex,
      partitionKey: {
        name: 'location_id',
        type: AttributeType.STRING,
      },
      projectionType: ProjectionType.INCLUDE,
      nonKeyAttributes: [
        'street',
        'district',
        'description',
      ],
    });


    const scraper = new GfaFunctionWithInvokeTask(this, 'scraper', {
      name: 'scraper',
      outputPath: '$.Payload'
    });

    const saveEvents = new GfaFunctionWithInvokeTask(this, 'save-events', {
      name: 'save-events',
      environment: {
        EVENTS_TABLE: this.eventsTable.tableName
      }
    });
    this.eventsTable.grantWriteData(saveEvents.handler);

    const alertTask = new SnsPublish(this, 'Data ingestion alert', {
      topic: props.alertTopic,
      message: TaskInput.fromDataAt('$.Cause'),
      subject: 'GFA: Data ingestion alert'
    });
    const scrapeAndSaveFlow = new StateMachine(this, 'scrape-and-save', {
      definition: scraper.task.addCatch(alertTask)
        .next(saveEvents.task.addCatch(alertTask)),
      timeout: Duration.minutes(5)
    });

    new Rule(this, 'scrape-and-save-scheduled-execution', {
      schedule: Schedule.expression('cron(0 0 1 * ? *)'),
      targets: [new SfnStateMachine(scrapeAndSaveFlow)],
    });
  }
}
