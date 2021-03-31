#!/usr/bin/env node
import { App } from '@aws-cdk/core';
import { GbgFarligtAvfallStack } from '../lib/gfa-stack';

const app = new App();
const gfaStack = new GbgFarligtAvfallStack(app, 'GbgFarligtAvfallStack');
