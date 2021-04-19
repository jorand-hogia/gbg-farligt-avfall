import { NestedStack, NestedStackProps } from '@aws-cdk/aws-cloudformation';
import { Construct } from '@aws-cdk/core';
import { IBucket } from '@aws-cdk/aws-s3';
import { ITable } from '@aws-cdk/aws-dynamodb';
import { ITopic, Topic } from '@aws-cdk/aws-sns';
import { Effect, PolicyStatement } from '@aws-cdk/aws-iam';
import { Rule, Schedule } from '@aws-cdk/aws-events';
import { LambdaFunction } from '@aws-cdk/aws-events-targets';
import { Alarm, ComparisonOperator } from '@aws-cdk/aws-cloudwatch';
import { SnsAction } from '@aws-cdk/aws-cloudwatch-actions';
import { LambdaEndpoint } from './gfa-api-stack';
import { GfaFunction } from './function/gfa-function';

interface NotifyStackProps extends NestedStackProps {
    eventsTable: ITable,
    alertTopic: ITopic
}

export class NotifyStack extends NestedStack {

    public readonly subscribeEndpoint: LambdaEndpoint;

    constructor(scope: Construct, id: string, props: NotifyStackProps) {
        super(scope, id, props);

        const arrivalToday = new Topic(this, 'gfa-today-topic');

        const notify = new GfaFunction(this, 'notify', {
            name: 'notify',
            environment: {
                EVENTS_TABLE: props.eventsTable.tableName,
                TODAY_TOPIC: arrivalToday.topicArn,
            }
        });
        props.eventsTable.grantReadData(notify.handler);
        arrivalToday.grantPublish(notify.handler);
        new Rule(this, 'gfa-notify-scheduled-execution', {
            schedule: Schedule.expression('cron(0 3 * * ? *)'),
            targets: [new LambdaFunction(notify.handler)]
        });
        new Alarm(this, 'Notify alert', {
            metric: notify.handler.metricErrors(),
            threshold: 0,
            comparisonOperator: ComparisonOperator.GREATER_THAN_THRESHOLD,
            evaluationPeriods: 1,
            datapointsToAlarm: 1
        }).addAlarmAction(new SnsAction(props.alertTopic));

        const subscribe = new GfaFunction(this, 'subscribe', {
            name: 'subscribe',
            environment: {
                TODAY_TOPIC: arrivalToday.topicArn,
            }
        });
        subscribe.handler.addToRolePolicy(new PolicyStatement({
            effect: Effect.ALLOW,
            actions: [ 'sns:Subscribe' ],
            resources: [ arrivalToday.topicArn ]
        }));
        this.subscribeEndpoint = {
            lambda: subscribe.handler,
            resource: 'subscriptions',
            methods: ['PUT']
        };
    }
}
