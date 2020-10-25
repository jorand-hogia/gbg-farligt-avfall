import { Construct, Duration } from '@aws-cdk/core';
import { Function, Runtime, CfnParametersCode } from '@aws-cdk/aws-lambda';
import { NestedStack, NestedStackProps } from '@aws-cdk/aws-cloudformation';
import { LambdaInvoke } from '@aws-cdk/aws-stepfunctions-tasks';
import { Parallel, StateMachine } from '@aws-cdk/aws-stepfunctions';
import { Secret } from "@aws-cdk/aws-secretsmanager";
import { IBucket } from '@aws-cdk/aws-s3';

export interface IngestionStackProps extends NestedStackProps {
  scraperCode: CfnParametersCode,
  saveEventsCode: CfnParametersCode,
  preProcessStopsCode: CfnParametersCode,
  saveStopsCode: CfnParametersCode,
  stopsBucket: IBucket,
  stopsPath: string,
}

export class IngestionStack extends NestedStack {
  contructor(scope: Construct, id: string, props: IngestionStackProps) {
    const scraper = new Function(this, 'gfa-scraper', {
      code: props.scraperCode,
      handler: 'doesnt.matter',
      runtime: Runtime.PROVIDED,
      timeout: Duration.seconds(10),
    });
    const scrapeTask = new LambdaInvoke(this, 'gfa-task-scrape', {
      lambdaFunction: scraper,
      outputPath: '$.Payload'
    });

    const saveEvents = new Function(this, 'gfa-save-events', {
      code: props.saveEventsCode,
      handler: 'doesnt.matter',
      runtime: Runtime.PROVIDED,
      timeout: Duration.seconds(10)
    });
    const saveEventsTask = new LambdaInvoke(this, 'gfa-task-save-events', {
      lambdaFunction: saveEvents
    });

    const preProcessStops = new Function(this, 'gfa-pre-process-stops', {
      code: props.preProcessStopsCode,
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
      code: props.saveStopsCode,
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
