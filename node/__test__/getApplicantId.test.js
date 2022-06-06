'use strict';

const assert = require('assert');

const { getApplicantId } = require('..');

describe('getApplicantId', () => {
  it('should work', async () => {
    assert.strictEqual(getApplicantId('1', '2', '3', '4'), '3d66ff22fd43e3b37d3a4a06322cc636');
  });
});
