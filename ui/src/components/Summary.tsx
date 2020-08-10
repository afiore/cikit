import React from 'react';
import { Summary } from '../dtos';

interface Props {
    summary: Summary
}

interface Distribution {
    failed: number;
    skipped: number;
    successful: number;
}

export function allocateWithin(summary: Summary, slots: number): Distribution {
    let factor = slots / summary.tests;
    let failed = Math.ceil((summary.errors + summary.failures) * factor);
    let skipped = Math.ceil(summary.skipped * factor);
    let successful = slots - failed - skipped;
    return { failed, skipped, successful }
}


export const SummaryFragment = (props: Props) => {
    let dist = allocateWithin(props.summary, 24);
    return (<>
        <p>{props.summary.failures} failures, {props.summary.errors} errors, {props.summary.skipped} skipped </p>
        {(props.summary.tests < 1) ? (<></>) : (
            <div className="pure-g summary-bar">
                <div className={"failed pure-u-" + dist.failed + "-24"}></div>
                <div className={"skipped pure-u-" + dist.skipped + "-24"} ></div>
                <div className={"successful pure-u-" + dist.successful + "-24"}></div>
            </div>)
        }

    </>)
}
