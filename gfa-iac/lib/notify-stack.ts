import { NestedStack, NestedStackProps } from '@aws-cdk/aws-cloudformation';
import { Construct } from '@aws-cdk/core';
import { ITable } from '@aws-cdk/aws-dynamodb';
import { ITopic, Topic } from '@aws-cdk/aws-sns';
import { Rule, Schedule } from '@aws-cdk/aws-events';
import { LambdaFunction } from '@aws-cdk/aws-events-targets';
import { Alarm, ComparisonOperator } from '@aws-cdk/aws-cloudwatch';
import { SnsAction } from '@aws-cdk/aws-cloudwatch-actions';
import { GfaFunction } from './function/gfa-function';

interface NotifyStackProps extends NestedStackProps {
    eventsTable: ITable,
    subscriptionsTable: ITable,
    apiKey: string,
    emailDomain: string,
    alertTopic: ITopic
}

export class NotifyStack extends NestedStack {
    constructor(scope: Construct, id: string, props: NotifyStackProps) {
        super(scope, id, props);

        const notify = new GfaFunction(this, 'notify', {
            name: 'notify',
            environment: {
                EVENTS_TABLE: props.eventsTable.tableName,
                SUBSCRIPTIONS_TABLE: props.subscriptionsTable.tableName,
                SENDGRID_API_KEY: props.apiKey,
                EMAIL_DOMAIN: props.emailDomain,
            }
        });
        props.eventsTable.grantReadData(notify.handler);
        props.subscriptionsTable.grantReadData(notify.handler);

        new Rule(this, 'notify-scheduled-execution', {
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
    }
}
