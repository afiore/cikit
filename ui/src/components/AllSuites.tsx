import React from "react";
import { TestSuite } from "../dtos";
import { showDuration } from "../utils";

interface Props {
    all: TestSuite[]
}

export class Component extends React.Component<Props, any> {
    render() {
        return (
            <section>
                <h2>All suites</h2>
                <table className="pure-table">
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Tests</th>
                            <th>Failed</th>
                            <th>Skipped</th>
                            <th>Duration</th>
                        </tr>
                    </thead>
                    {this.props.all.map(suite => {
                        return (
                            <tbody key={suite.name + "-tbody"}>
                                <tr key={suite.name + "-all-tests"} className={"pure-table-odd"}>
                                    <td>{suite.name}</td>
                                    <td>{suite.tests}</td>
                                    <td>{suite.skipped}</td>
                                    <td>{suite.failures}</td>
                                    <td>{showDuration(suite.time)}</td>
                                </tr>
                            </tbody>
                        )
                    })
                    }
                </table >
            </section>
        );
    }
}
