#!/usr/bin/env node
import { App } from '@aws-cdk/core';
import { GbgFarligtAvfallStack } from '../lib/gfa-stack';

const app = new App();

const artifactsBucketName = app.node.tryGetContext('artifactsBucketName');
const version = app.node.tryGetContext('version');

const hostedZoneId = app.node.tryGetContext('hostedZoneId');
const domainName = app.node.tryGetContext('domainName');
const adminEmail = app.node.tryGetContext('adminEmail');

const gfaStack = new GbgFarligtAvfallStack(app, 'GbgFarligtAvfallStack', {
  artifactsBucketName,
  version,
  hostedZoneId,
  domainName,
  adminEmail
});
