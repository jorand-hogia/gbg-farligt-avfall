import { App, CfnOutput, Stack, StackProps } from '@aws-cdk/core';
import { Table, AttributeType, BillingMode } from '@aws-cdk/aws-dynamodb';
import { Bucket } from '@aws-cdk/aws-s3';
import { Topic } from '@aws-cdk/aws-sns';
import { EmailSubscription} from '@aws-cdk/aws-sns-subscriptions';
import { IngestionStack } from './gfa-ingestion-stack';
import { ApiStack } from './gfa-api-stack';
import { WebStack } from './gfa-web-stack';
import { NotifyStack } from './gfa-notify-stack';
import { GfaFunction } from './function/gfa-function';
import { SendGridDomainVerifier } from './sendgrid/domain-verifier';

export class GbgFarligtAvfallStack extends Stack {

  constructor(app: App, id: string) {
    super(app, id);

    const eventsDb = new Table(this, 'gfa-events-db', {
      partitionKey: { name: 'event_date', type: AttributeType.STRING },
      sortKey: { name: 'location_id', type: AttributeType.STRING },
      billingMode: BillingMode.PAY_PER_REQUEST,
    });

    const stopsS3Path = 'stops.json';
    const stopsBucket = new Bucket(this, 'gfa-stops-bucket');

    const alertTopic = new Topic(this, 'gfa-admin-alert');
    const adminEmail = app.node.tryGetContext('adminEmail');
    if (adminEmail) {
      alertTopic.addSubscription(new EmailSubscription(adminEmail));
    }

    new IngestionStack(this, 'gfa-ingestion-stack', {
      stopsBucket: stopsBucket,
      stopsPath: stopsS3Path,
      eventsTable: eventsDb,
      alertTopic,
    });

    const getStops = new GfaFunction(this, 'get-stops', {
        name: 'get-stops',
        environment: {
            STOPS_BUCKET: stopsBucket.bucketName,
            STOPS_PATH: stopsS3Path,
        }
    });
    stopsBucket.grantRead(getStops.handler, stopsS3Path);

    const notifyStack = new NotifyStack(this, 'gfa-notify-stack', {
      eventsTable: eventsDb,
      alertTopic,
    });

    const webStack = new WebStack(this, 'gfa-web-stack');
    const apiStack = new ApiStack(this, 'gfa-api-stack', {
      lambdaEndpoints: [
        {
          lambda: getStops.handler,
          resource: 'stops',
          methods: ['GET']
        },
        notifyStack.subscribeEndpoint,
      ]
    });

    const sendgridApiKey = app.node.tryGetContext('sendgridApiKey');
    const hostedZoneId = app.node.tryGetContext('hostedZoneId');
    const domainName = app.node.tryGetContext('domainName');
    new SendGridDomainVerifier(this, 'gfa-sendgrid-verifier', {
      hostedZoneId,
      domainName,
      apiKey: sendgridApiKey,
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
