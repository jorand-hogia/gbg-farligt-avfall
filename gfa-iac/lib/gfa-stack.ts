import { App, Stack, StackProps, CfnParameter } from '@aws-cdk/core';
import { Table, AttributeType, BillingMode } from '@aws-cdk/aws-dynamodb';
import { Bucket } from '@aws-cdk/aws-s3';
import { IngestionStack, IngestionStackProps } from './gfa-ingestion-stack';

interface GbgFarligtAvfallStackProps extends StackProps {
  artifactsBucketName: string,
  version: string,
}

export class GbgFarligtAvfallStack extends Stack {

  constructor(app: App, id: string, props: GbgFarligtAvfallStackProps) {
    super(app, id, props);

    const artifactsBucket = Bucket.fromBucketName(this, 'artifactsBucket', props.artifactsBucketName);

    const eventsDb = new Table(this, 'gfa-events-db', {
      partitionKey: { name: 'event-date', type: AttributeType.STRING },
      sortKey: { name: 'location-id', type: AttributeType.STRING },
      billingMode: BillingMode.PAY_PER_REQUEST,
    });
    const stopsS3Path = 'stops.json';
    const stopsBucket = new Bucket(this, 'gfa-stops-bucket');

    const ingestionStackProps: IngestionStackProps = {
      version: props.version,
      artifactsBucket: artifactsBucket,
      stopsBucket: stopsBucket,
      stopsPath: stopsS3Path
    }
    const ingestionStack = new IngestionStack(this, 'gfa-ingestion-stack', ingestionStackProps);

  }
}
