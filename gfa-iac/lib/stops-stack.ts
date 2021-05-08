import { Construct, NestedStack } from "@aws-cdk/core";
import { HttpApi, HttpMethod } from '@aws-cdk/aws-apigatewayv2';
import { LambdaProxyIntegration } from '@aws-cdk/aws-apigatewayv2-integrations';
import { GfaFunction } from './function/gfa-function';
import { ITable } from "@aws-cdk/aws-dynamodb";

export interface StopsStackProps {
    api: HttpApi,
    eventsTable: ITable,
    locationIndex: string,
}

export class StopsStack extends NestedStack {
    constructor(scope: Construct, id: string, props: StopsStackProps) {
        super(scope, id);

        const getStops = new GfaFunction(this, 'get-stops', {
            name: 'get-stops',
            environment: {
                EVENTS_TABLE: props.eventsTable.tableName,
                LOCATION_INDEX: props.locationIndex,
            }
        });
        props.eventsTable.grantReadData(getStops.handler);

        props.api.addRoutes({
            path: '/stops',
            methods: [ HttpMethod.GET ],
            integration: new LambdaProxyIntegration({
                handler: getStops.handler,
            }),
        });
    }
}
