import { App, CfnOutput, Stack, StackProps } from '@aws-cdk/core';
import { Table, AttributeType, BillingMode } from '@aws-cdk/aws-dynamodb';
import { Bucket } from '@aws-cdk/aws-s3';
import { Topic } from '@aws-cdk/aws-sns';
import { EmailSubscription} from '@aws-cdk/aws-sns-subscriptions';
import { IngestionStack } from './gfa-ingestion-stack';
import { ApiStack } from './gfa-api-stack';
import { WebStack } from './gfa-web-stack';
import { NotifyStack } from './gfa-notify-stack';
import { functionCreator } from './function-creator';

interface GbgFarligtAvfallStackProps extends StackProps {
  artifactsBucketName: string,
  version: string,
  hostedZoneId: string,
  domainName: string,
  adminEmail: string,
}

export class GbgFarligtAvfallStack extends Stack {

  constructor(app: App, id: string, props: GbgFarligtAvfallStackProps) {
    super(app, id, props);

    const artifactsBucket = Bucket.fromBucketName(this, 'artifactsBucket', props.artifactsBucketName);

    const eventsDb = new Table(this, 'gfa-events-db', {
      partitionKey: { name: 'event_date', type: AttributeType.STRING },
      sortKey: { name: 'location_id', type: AttributeType.STRING },
      billingMode: BillingMode.PAY_PER_REQUEST,
    });

    const stopsS3Path = 'stops.json';
    const stopsBucket = new Bucket(this, 'gfa-stops-bucket');

    const alertTopic = new Topic(this, 'gfa-admin-alert');
    if (props.adminEmail) {
      alertTopic.addSubscription(new EmailSubscription(props.adminEmail));
    }

    const ingestionStack = new IngestionStack(this, 'gfa-ingestion-stack', {
      version: props.version,
      artifactsBucket: artifactsBucket,
      stopsBucket: stopsBucket,
      stopsPath: stopsS3Path,
      eventsTable: eventsDb,
      alertTopic,
    });


    const newFunction = functionCreator(artifactsBucket, props.version);
    const getStops = newFunction(this, 'get-stops', {
        environment: {
            STOPS_BUCKET: stopsBucket.bucketName,
            STOPS_PATH: stopsS3Path,
        }
    });
    stopsBucket.grantRead(getStops, stopsS3Path);

    const notifyStack = new NotifyStack(this, 'gfa-notify-stack', {
      version: props.version,
      artifactsBucket: artifactsBucket,
      eventsTable: eventsDb,
      alertTopic,
    });

    const webStack = new WebStack(this, 'gfa-web-stack', {
      hostedZoneId: props.hostedZoneId,
      domainName: props.domainName
    });
    const apiStack = new ApiStack(this, 'gfa-api-stack', {
      hostedZoneId: props.hostedZoneId,
      domainName: props.domainName,
      lambdaEndpoints: [
        {
          lambda: getStops,
          resource: 'stops',
          methods: ['GET']
        },
        notifyStack.subscribeEndpoint,
      ]
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
