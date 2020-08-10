import React from 'react';
import { render } from '@testing-library/react';
import { allocateWithin } from './Summary';

test('renders learn react link', () => {
    let summary = {
        time: 0,
        tests: 20,
        failures: 2,
        errors: 1,
        skipped: 5
    };
    let distribution = allocateWithin(summary, 24);
    expect(distribution.failed).toBe(4);
    expect(distribution.skipped).toBe(6);
    expect(distribution.successful).toBe(14);
});
