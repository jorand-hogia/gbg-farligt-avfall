import { Construct, Duration } from '@aws-cdk/core';
import { NestedStack, NestedStackProps } from '@aws-cdk/aws-cloudformation';
import { Parallel, StateMachine, TaskInput } from '@aws-cdk/aws-stepfunctions';
import { SnsPublish } from '@aws-cdk/aws-stepfunctions-tasks';
import { IBucket } from '@aws-cdk/aws-s3';
import { ITable } from '@aws-cdk/aws-dynamodb';
import { ITopic } from '@aws-cdk/aws-sns';
import { functionWithInvokeCreator } from './function-creator';

export interface IngestionStackProps extends NestedStackProps {
  version: string,
  artifactsBucket: IBucket,
  stopsBucket: IBucket,
  stopsPath: string,
  eventsTable: ITable,
  alertTopic: ITopic, 
}

export class IngestionStack extends NestedStack {

  constructor(scope: Construct, id: string, props: IngestionStackProps) {
    super(scope, id, props);

    const functionWithInvokeTask = functionWithInvokeCreator(props.artifactsBucket, props.version);

    const [scraper, invokeScraper] = functionWithInvokeTask(this, 'scraper', {
      outputPath: '$.Payload'
    });

    const [saveEvents, invokeSaveEvents] = functionWithInvokeTask(this, 'save-events', {
      environment: {
        EVENTS_TABLE: props.eventsTable.tableName, 
      }
    });
    props.eventsTable.grantWriteData(saveEvents);

    const [saveStops, invokeSaveStops] = functionWithInvokeTask(this, 'save-stops', {
      environment: {
        STOPS_BUCKET: props.stopsBucket.bucketName,
        STOPS_PATH: props.stopsPath,
      }
    });
    props.stopsBucket.grantWrite(saveStops, props.stopsPath);

    const alertTask = new SnsPublish(this, 'Data ingestion alert', {
            topic: props.alertTopic,
            message: TaskInput.fromDataAt('$.error'),
            subject: 'GFA: Data ingestion alert'
    });
    const scrapeAndSaveFlow = new StateMachine(this, 'gfa-scrape-and-save', {
      definition: invokeScraper 
        .addCatch(alertTask)
        .next(new Parallel(this, 'process-scrape-results', {})
          .branch(invokeSaveEvents)
          .branch(invokeSaveStops)
          .addCatch(alertTask)
        ),
      timeout: Duration.minutes(5)
    });
  }
}
