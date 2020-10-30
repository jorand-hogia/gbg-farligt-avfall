import { Construct, Duration } from '@aws-cdk/core';
import { NestedStack, NestedStackProps } from '@aws-cdk/aws-cloudformation';
import { Parallel, StateMachine } from '@aws-cdk/aws-stepfunctions';
import { Secret } from "@aws-cdk/aws-secretsmanager";
import { IBucket } from '@aws-cdk/aws-s3';
import { functionWithInvokeCreator } from './function-creator';

export interface IngestionStackProps extends NestedStackProps {
  version: string,
  artifactsBucket: IBucket,
  stopsBucket: IBucket,
  stopsPath: string,
}

export class IngestionStack extends NestedStack {

  constructor(scope: Construct, id: string, props: IngestionStackProps) {
    super(scope, id, props);

    const functionWithInvokeTask = functionWithInvokeCreator(props.artifactsBucket, props.version);

    const [scraper, invokeScraper] = functionWithInvokeTask(this, 'scraper', {
      outputPath: '$.Payload'
    });
    const [saveEvents, invokeSaveEvents] = functionWithInvokeTask(this, 'save-events');
    const [preProcessStops, invokePreprocessStops] = functionWithInvokeTask(this, 'preprocess-stops', {
      timeout: Duration.seconds(30),
      environment: {
        GEOCODING_API_KEY: `${Secret.fromSecretName(this, 'geocoding-api-key', 'mapquest-api-key').secretValue}`
      },
      outputPath: '$.Payload',
    });
    const [saveStops, invokeSaveStops] = functionWithInvokeTask(this, 'save-stops', {
      environment: {
        STOPS_BUCKET: props.stopsBucket.bucketName,
        STOPS_PATH: props.stopsPath,
      }
    });
    props.stopsBucket.grantWrite(saveStops, props.stopsPath);

    const scrapeAndSaveFlow = new StateMachine(this, 'gfa-scrape-and-save', {
      definition: invokeScraper 
        .next(new Parallel(this, 'process-scrape-results', {})
          .branch(invokeSaveEvents)
          .branch(invokePreprocessStops
            .next(invokeSaveStops))
        ),
      timeout: Duration.minutes(5)
    });
  }
}
