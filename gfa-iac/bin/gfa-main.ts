#!/usr/bin/env node
import { App } from '@aws-cdk/core';
import { GbgFarligtAvfallStack } from '../lib/main-stack';

const app = new App();
const gfaStack = new GbgFarligtAvfallStack(app, 'GbgFarligtAvfallStack');
