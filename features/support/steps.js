const { Given, When, Then } = require("cucumber");
const { expect } = require("chai");

 Given('I have the environment prepared.', function () {
     this.prepareEnvironment();
 });

 When('{string} creates an asset with metadata: {string} and {string} emissions with the amount: {int} Grams per kilo CO2 emitted from date: {int}.', function (string, string, string, int, int) {
     // this.blastAsset(string, string, string, int, int);
     this.addTo("I");
 });

 Then('the result should be the number {string}', function (string) {
   expect(this.result).to.eql(string);
 });
