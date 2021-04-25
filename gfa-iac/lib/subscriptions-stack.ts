import { NestedStack, NestedStackProps } from '@aws-cdk/aws-cloudformation';
import { Construct } from "@aws-cdk/core";
import { Table, AttributeType, BillingMode } from '@aws-cdk/aws-dynamodb';
import { GfaFunction } from './function/gfa-function';
import { RestApi, LambdaIntegration, Cors } from '@aws-cdk/aws-apigateway';

export interface SubscriptionsStackProps extends NestedStackProps {
    api: RestApi,
}

export class SubscriptionStack extends NestedStack {
    constructor(scope: Construct, id: string, props: SubscriptionsStackProps) {
        super(scope, id);

        const subscriptionsDb = new Table(this, 'subscriptions-db', {
            partitionKey: { name: 'email', type: AttributeType.STRING },
            sortKey: { name: 'location_id', type: AttributeType.STRING },
            billingMode: BillingMode.PAY_PER_REQUEST,
            timeToLiveAttribute: 'ttl'
        });
        subscriptionsDb.addGlobalSecondaryIndex({
            indexName: 'byAuthToken',
            partitionKey: { name: 'auth_token', type: AttributeType.STRING }
        });

        const addSubscription = new GfaFunction(this, 'addSubscription', {
            name: 'add-subscription',
            environment: {
                SUBSCRIPTIONS_TABLE: subscriptionsDb.tableName
            }
        });
        subscriptionsDb.grantReadWriteData(addSubscription.handler);

        const verifySubscription = new GfaFunction(this, 'verifySubscription', {
            name: 'verify-subscription',
            environment: {
                SUBSCRIPTIONS_TABLE: subscriptionsDb.tableName
            }
        });
        subscriptionsDb.grantReadWriteData(verifySubscription.handler);

        const addSubscriptionIntegration = new LambdaIntegration(addSubscription.handler, {
            proxy: true,
        });
        const verifySubscriptionIntegration = new LambdaIntegration(verifySubscription.handler, {
            proxy: true,
        });

        const subscriptionsResource = props.api.root.addResource('subscriptions');
        subscriptionsResource.addCorsPreflight({
            allowOrigins: Cors.ALL_ORIGINS,
            allowMethods: ['PUT'],
            allowHeaders: ['Content-Type', 'Accept']
        });
        subscriptionsResource.addMethod('PUT', addSubscriptionIntegration);

        const verifyResource = subscriptionsResource.addResource('verify');
        verifyResource.addCorsPreflight({
            allowOrigins: Cors.ALL_ORIGINS,
            allowMethods: ['POST'],
            allowHeaders: ['Content-Typ', 'Accept']
        });
        verifyResource.addMethod('POST', verifySubscriptionIntegration);
    }

}
