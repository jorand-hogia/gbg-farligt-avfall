import { App, CfnOutput, RemovalPolicy, Stack, StackProps } from '@aws-cdk/core';
import { Table, AttributeType, BillingMode } from '@aws-cdk/aws-dynamodb';
import { Bucket } from '@aws-cdk/aws-s3';
import { Topic } from '@aws-cdk/aws-sns';
import { EmailSubscription} from '@aws-cdk/aws-sns-subscriptions';
import { IngestionStack } from './ingestion-stack';
import { ApiStack } from './api-stack';
import { WebStack } from './web-stack';
import { NotifyStack } from './notify-stack';
import { SendGridDomainVerifier } from './sendgrid/domain-verifier';
import { SubscriptionStack } from './subscriptions-stack';
import { StopsStack } from './stops-stack';

export interface GbgFarligtAvfallStackProps extends StackProps {
  webCertParameterName: string
}

export class GbgFarligtAvfallStack extends Stack {

  constructor(app: App, id: string, props: GbgFarligtAvfallStackProps) {
    super(app, id, props);

    const eventsDb = new Table(this, 'events-db', {
      partitionKey: { name: 'event_date', type: AttributeType.STRING },
      sortKey: { name: 'location_id', type: AttributeType.STRING },
      billingMode: BillingMode.PAY_PER_REQUEST,
      removalPolicy: RemovalPolicy.DESTROY
    });

    const stopsS3Path = 'stops.json';
    const stopsBucket = new Bucket(this, 'stops-bucket', {
      removalPolicy: RemovalPolicy.DESTROY
    });

    const webStack = new WebStack(this, 'web-stack', {
      webCertParameterName: props.webCertParameterName
    });
    const apiStack = new ApiStack(this, 'api-stack');

    const alertTopic = new Topic(this, 'admin-alert');
    const adminEmail = app.node.tryGetContext('adminEmail');
    if (adminEmail) {
      alertTopic.addSubscription(new EmailSubscription(adminEmail));
    }

    new IngestionStack(this, 'ingestion-stack', {
      stopsBucket: stopsBucket,
      stopsPath: stopsS3Path,
      eventsTable: eventsDb,
      alertTopic,
    });

    new StopsStack(this, 'stops-stack', {
      api: apiStack.api,
      stopsBucket: stopsBucket,
      stopsPath: stopsS3Path
    })

    const domainName = app.node.tryGetContext('domainName');
    const sendgridApiKey = app.node.tryGetContext('sendgridApiKey');
    const subscriptionsStack = new SubscriptionStack(this, 'subscription-stack', {
      api: apiStack.api,
      verifyUrl: `https://${webStack.externalDomain}/verify`,
      emailDomain: domainName,
      apiKey: sendgridApiKey,
    });

    new NotifyStack(this, 'notify-stack', {
      eventsTable: eventsDb,
      subscriptionsTable: subscriptionsStack.subscriptionsDb, 
      apiKey: sendgridApiKey,
      emailDomain: domainName,
      alertTopic,
    });

    const hostedZoneId = app.node.tryGetContext('hostedZoneId');
    new SendGridDomainVerifier(this, 'sendgrid-verifier', {
      hostedZoneId,
      domainName,
      apiKey: sendgridApiKey,
    });

    new CfnOutput(this, 'WebBucket', {
      value: webStack.webHostingBucketName,
    });
    new CfnOutput(this, 'WebDistributionId', {
      value: webStack.webDistributionId,
    });
    new CfnOutput(this, 'ApiUrl', {
      value: `https://${apiStack.externalDomain}`,
    });
    new CfnOutput(this, 'WebUrl', {
      value: `https://${webStack.externalDomain}`,
    });
  }
}
