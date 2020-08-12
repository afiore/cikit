import React from 'react';
import './App.css';
import * as FailedSuites from './components/FailedSuites';
import * as AllSuites from './components/AllSuites';
import { FailedTestSuite, GithubContext, Summary, TestSuite } from './dtos';
import { SummaryFragment } from './components/Summary';
import { GithubContextFragment } from './components/GithubContext';

interface AppProps {
  datasetUri: string
}
interface AppState {
  failed: FailedTestSuite[];
  all: TestSuite[];
  summary: Summary;
  githubEvent: GithubContext | null;
}

class App extends React.Component<AppProps, AppState> {
  constructor(props: any) {
    super(props);

    this.state = {
      summary: {
        time: 0,
        tests: 0,
        failures: 0,
        errors: 0,
        skipped: 0,
      },
      failed: [],
      all: [],
      githubEvent: null
    };
  }

  componentDidMount() {
    fetch(this.props.datasetUri)
      .then(response => response.json())
      .then(result => this.setState({
        summary: result.summary,
        failed: result.failed,
        all: result.allSuites,
        githubEvent: result.githubEvent
      }));
  }

  render() {
    return (
      <section>
        <GithubContextFragment context={this.state.githubEvent} />
        <SummaryFragment summary={this.state.summary} />
        {(this.state.failed.length > 0) ? <FailedSuites.Component failed={this.state.failed} /> : (<></>)}
        <AllSuites.Component all={this.state.all} />
      </section>
    );
  }
}

export default App;
