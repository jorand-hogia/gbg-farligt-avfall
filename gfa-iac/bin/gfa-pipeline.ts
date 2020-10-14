#!/usr/bin/env node
import * as dotenv from "dotenv";
import 'source-map-support/register';
import { App } from '@aws-cdk/core';
import { GbgFarligtAvfallPipelineStack } from '../lib/gfa-pipeline-stack';
import { GbgFarligtAvfallStack } from '../lib/gfa-stack';

dotenv.config();

if (!process.env.GITHUB_TOKEN) {
  console.log("No Github Token present");
}

const app = new App();

const gfaStack = new GbgFarligtAvfallStack(app, 'GbgFarligtAvfallStack')
new GbgFarligtAvfallPipelineStack(app, 'GbgFarligtAvfallPipelineStack', {
    scraperCode: gfaStack.scraperCode,
    eventsCode: gfaStack.eventsCode,
    repoOwner: 'Dunklas', // TODO: Parameterize!
    repoName: 'gbg-farligt-avfall', // TODO: Parameterize!
    githubToken: process.env.GITHUB_TOKEN || ''
})

app.synth();
