import { FailedTestCase, } from '../dtos';
import { showDuration } from '../utils';
import React from 'react';

interface Props {
    testCase: FailedTestCase
}

interface State {
    isExpanded: boolean
}

export class Component extends React.Component<Props, State> {
    constructor(props: Props) {
        super(props);
        this.state = {
            isExpanded: false
        }
    }
    handleOnClick = (testCaseName: string) => {
        this.setState((prevState, _) => {
            return { isExpanded: !prevState.isExpanded }
        });
    }
    render() {
        let testCase = this.props.testCase;
        let state = this.state;

        return (
            <tr key={testCase.name} className="failedtests">
                <td colSpan={4}>
                    <button title="hide/show failure details" onClick={() => this.handleOnClick(testCase.name)}>{state.isExpanded ? "-" : "+"}</button>
                    {testCase.name}
                    {(state.isExpanded) ?
                        (<section>
                            <pre>
                                {testCase.failure.stackTrace}
                            </pre>
                        </section>) : null
                    }
                </td>
                <td>{showDuration(testCase.time)}</td>
            </tr>)
    }
}

