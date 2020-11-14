import { App, CfnOutput, Stack, StackProps } from '@aws-cdk/core';
import { Table, AttributeType, BillingMode } from '@aws-cdk/aws-dynamodb';
import { Bucket } from '@aws-cdk/aws-s3';
import { IngestionStack } from './gfa-ingestion-stack';
import { ApiStack } from './gfa-api-stack';
import { WebStack } from './gfa-web-stack';

interface GbgFarligtAvfallStackProps extends StackProps {
  artifactsBucketName: string,
  version: string,
  hostedZoneId: string,
  domainName: string,
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

    const ingestionStack = new IngestionStack(this, 'gfa-ingestion-stack', {
      version: props.version,
      artifactsBucket: artifactsBucket,
      stopsBucket: stopsBucket,
      stopsPath: stopsS3Path,
      eventsTable: eventsDb,
    });

    const webStack = new WebStack(this, 'gfa-web-stack');
    const apiStack = new ApiStack(this, 'gfa-api-stack', {
      version: props.version,
      artifactsBucket: artifactsBucket,
      stopsBucket: stopsBucket,
      stopsPath: stopsS3Path,
      hostedZoneId: props.hostedZoneId,
      domainName: props.domainName,
    });

    new CfnOutput(this, 'WebBucket', {
      value: webStack.webHostingBucketName,
    });
    new CfnOutput(this, 'ApiUrl', {
      value: apiStack.apiUrl,
    });
    new CfnOutput(this, 'WebUrl', {
      value: webStack.webUrl,
    });
    new CfnOutput(this, 'WebDistributionId', {
      value: webStack.webDistributionId,
    })
  }
}
