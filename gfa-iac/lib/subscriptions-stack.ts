import { NestedStack, NestedStackProps } from '@aws-cdk/aws-cloudformation';
import { Construct } from "@aws-cdk/core";
import { Table, AttributeType, BillingMode } from '@aws-cdk/aws-dynamodb';
import { GfaFunction } from './function/gfa-function';
import { HttpApi, HttpMethod } from '@aws-cdk/aws-apigatewayv2';
import { LambdaProxyIntegration } from '@aws-cdk/aws-apigatewayv2-integrations';

export interface SubscriptionsStackProps extends NestedStackProps {
    api: HttpApi,
    verifyUrl: string,
    emailDomain: string,
    apiKey: string,
    eventsTable: Table,
    locationIndex: string,
}

export class SubscriptionStack extends NestedStack {

    public readonly subscriptionsDb: Table;

    constructor(scope: Construct, id: string, props: SubscriptionsStackProps) {
        super(scope, id);

        this.subscriptionsDb = new Table(this, 'subscriptions-db', {
            partitionKey: { name: 'email', type: AttributeType.STRING },
            sortKey: { name: 'location_id', type: AttributeType.STRING },
            billingMode: BillingMode.PAY_PER_REQUEST,
            timeToLiveAttribute: 'ttl'
        });
        this.subscriptionsDb.addGlobalSecondaryIndex({
            indexName: 'byAuthToken',
            partitionKey: { name: 'auth_token', type: AttributeType.STRING }
        });
        this.subscriptionsDb.addGlobalSecondaryIndex({
            indexName: 'byUnsubscribeToken',
            partitionKey: { name: 'unsubscribe_token', type: AttributeType.STRING }
        })
        this.subscriptionsDb.addGlobalSecondaryIndex({
            indexName: 'byLocationId',
            partitionKey: { name: 'location_id', type: AttributeType.STRING },
            sortKey: { name: 'email', type: AttributeType.STRING },
        });

        const addSubscription = new GfaFunction(this, 'addSubscription', {
            name: 'add-subscription',
            environment: {
                SUBSCRIPTIONS_TABLE: this.subscriptionsDb.tableName,
                EVENTS_TABLE: props.eventsTable.tableName,
                LOCATION_INDEX: props.locationIndex,
                VERIFY_URL: props.verifyUrl,
                SENDGRID_API_KEY: props.apiKey,
                EMAIL_DOMAIN: props.emailDomain,
            },
        });
        this.subscriptionsDb.grantReadWriteData(addSubscription.handler);
        props.eventsTable.grantReadData(addSubscription.handler);

        const verifySubscription = new GfaFunction(this, 'verifySubscription', {
            name: 'verify-subscription',
            environment: {
                SUBSCRIPTIONS_TABLE: this.subscriptionsDb.tableName
            }
        });
        this.subscriptionsDb.grantReadWriteData(verifySubscription.handler);

        const removeSubscription = new GfaFunction(this, 'removeSubscription', {
            name: 'remove-subscription',
            environment: {
                SUBSCRIPTIONS_TABLE: this.subscriptionsDb.tableName,
            },
        });
        this.subscriptionsDb.grantReadWriteData(removeSubscription.handler);

        props.api.addRoutes({
            path: '/subscriptions',
            methods: [ HttpMethod.PUT ],
            integration: new LambdaProxyIntegration({
                handler: addSubscription.handler,
            }),
        });
        props.api.addRoutes({
            path: '/subscriptions',
            methods: [ HttpMethod.DELETE ],
            integration: new LambdaProxyIntegration({
                handler: removeSubscription.handler,
            }),
        });
        props.api.addRoutes({
            path: '/subscriptions/verify',
            methods: [ HttpMethod.POST ],
            integration: new LambdaProxyIntegration({
                handler: verifySubscription.handler,
            }),
        });
    }

}
