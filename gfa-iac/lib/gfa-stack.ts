import { App, CfnOutput, Stack, StackProps } from '@aws-cdk/core';
import { Table, AttributeType, BillingMode } from '@aws-cdk/aws-dynamodb';
import { Bucket } from '@aws-cdk/aws-s3';
import { Topic } from '@aws-cdk/aws-sns';
import { EmailSubscription} from '@aws-cdk/aws-sns-subscriptions';
import { IngestionStack } from './gfa-ingestion-stack';
import { ApiStack } from './gfa-api-stack';
import { WebStack } from './gfa-web-stack';
import { NotifyStack } from './gfa-notify-stack';
import { SendGridDomainVerifier } from './sendgrid/domain-verifier';
import { SubscriptionStack } from './gfa-subscriptions-stack';
import { StopsStack } from './gfa-stops-stack';

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

    new NotifyStack(this, 'gfa-notify-stack', {
      eventsTable: eventsDb,
      alertTopic,
    });

    const webStack = new WebStack(this, 'gfa-web-stack');
    const apiStack = new ApiStack(this, 'gfa-api-stack');

    const sendgridApiKey = app.node.tryGetContext('sendgridApiKey');
    const hostedZoneId = app.node.tryGetContext('hostedZoneId');
    const domainName = app.node.tryGetContext('domainName');
    new SendGridDomainVerifier(this, 'gfa-sendgrid-verifier', {
      hostedZoneId,
      domainName,
      apiKey: sendgridApiKey,
    });

    const stopsStack = new StopsStack(this, 'gfa-stops-stack', {
      api: apiStack.api,
      stopsBucket: stopsBucket,
      stopsPath: stopsS3Path
    })

    const subscriptionStack = new SubscriptionStack(this, 'gfa-subscription-stack', {
      api: apiStack.api
    });

    new CfnOutput(this, 'WebBucket', {
      value: webStack.webHostingBucketName,
    });
    new CfnOutput(this, 'ApiUrl', {
      value: apiStack.externalUrl || apiStack.api.url,
    });
    new CfnOutput(this, 'WebUrl', {
      value: webStack.webUrl,
    });
    new CfnOutput(this, 'WebDistributionId', {
      value: webStack.webDistributionId,
    })
  }
}
