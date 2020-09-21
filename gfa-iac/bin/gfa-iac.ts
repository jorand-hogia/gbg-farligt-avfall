#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from '@aws-cdk/core';
import { GfaIacStack } from '../lib/gfa-iac-stack';

const app = new cdk.App();
new GfaIacStack(app, 'GfaIacStack');
