import { Construct, Duration } from '@aws-cdk/core';
import { Function, Runtime, S3Code } from '@aws-cdk/aws-lambda';
import { NestedStack, NestedStackProps } from '@aws-cdk/aws-cloudformation';
import { LambdaInvoke } from '@aws-cdk/aws-stepfunctions-tasks';
import { Parallel, StateMachine } from '@aws-cdk/aws-stepfunctions';
import { Secret } from "@aws-cdk/aws-secretsmanager";
import { IBucket, Bucket } from '@aws-cdk/aws-s3';

export interface IngestionStackProps extends NestedStackProps {
  version: string,
  artifactsBucket: IBucket,
  stopsBucket: IBucket,
  stopsPath: string,
}

export class IngestionStack extends NestedStack {

  constructor(scope: Construct, id: string, props: IngestionStackProps) {
    super(scope, id, props);

    const scraper = new Function(this, 'gfa-scraper', {
      code: new S3Code(props.artifactsBucket, `gfa-scraper-${props.version}`),
      handler: 'doesnt.matter',
      runtime: Runtime.PROVIDED,
      timeout: Duration.seconds(10),
    });
    const scrapeTask = new LambdaInvoke(this, 'gfa-task-scrape', {
      lambdaFunction: scraper,
      outputPath: '$.Payload'
    });

    const saveEvents = new Function(this, 'gfa-save-events', {
      code: new S3Code(props.artifactsBucket, `gfa-save-events-${props.version}`),
      handler: 'doesnt.matter',
      runtime: Runtime.PROVIDED,
      timeout: Duration.seconds(10)
    });
    const saveEventsTask = new LambdaInvoke(this, 'gfa-task-save-events', {
      lambdaFunction: saveEvents
    });

    const preProcessStops = new Function(this, 'gfa-pre-process-stops', {
      code: new S3Code(props.artifactsBucket, `gfa-preprocess-stops-${props.version}`),
      handler: 'doesnt.matter',
      runtime: Runtime.PROVIDED,
      timeout: Duration.seconds(30),
      environment: {
        GEOCODING_API_KEY: `${Secret.fromSecretName(this, 'geocoding-api-key', 'mapquest-api-key').secretValue
          }`
      }
    });
    const preProcessStopsTask = new LambdaInvoke(this, 'gfa-task-pre-process-stops', {
      lambdaFunction: preProcessStops,
      outputPath: '$.Payload'
    });

    const saveStops = new Function(this, 'gfa-save-stops', {
      code: new S3Code(props.artifactsBucket, `gfa-save-stops-${props.version}`),
      handler: 'doesnt.matter',
      runtime: Runtime.PROVIDED,
      timeout: Duration.seconds(10),
      environment: {
        STOPS_BUCKET: props.stopsBucket.bucketName,
        STOPS_PATH: props.stopsPath,
      }
    });
    props.stopsBucket.grantWrite(saveStops, props.stopsPath);
    const saveStopsTask = new LambdaInvoke(this, 'gfa-task-save-stops', {
      lambdaFunction: saveStops,
    });

    const scrapeAndSaveFlow = new StateMachine(this, 'gfa-scrape-and-save', {
      definition: scrapeTask
        .next(new Parallel(this, 'process-scrape-results', {})
          .branch(saveEventsTask)
          .branch(preProcessStopsTask
            .next(saveStopsTask))
        ),
      timeout: Duration.minutes(5)
    });
  }
}
