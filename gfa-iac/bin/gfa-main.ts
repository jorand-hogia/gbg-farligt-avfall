#!/usr/bin/env node
import { App } from '@aws-cdk/core';
import { GbgFarligtAvfallStack } from '../lib/main-stack';
import { WebCertStack } from '../lib/web-cert-stack';

const app = new App();

// To use an ACM certificate for CloudFront, it must exist in us-east-1
// To enable usage in other regions, this stack puts the certificateArn in Parameter Store
new WebCertStack(app, 'GbgFarligtAvfallWebCertStack', {
    env: {
        region: 'us-east-1',
    },
});

new GbgFarligtAvfallStack(app, 'GbgFarligtAvfallStack');
