import React from 'react';
import { isNumericLiteral } from 'typescript';
import { GithubContext } from '../dtos';


interface Props {
    context: GithubContext | null;
}

export function GithubContextFragment(props: Props): JSX.Element {
    let ctx = props.context;
    if (ctx === null) {
        return <></>
    } else {
        let pr = ctx.pullRequest;
        let sender = ctx.sender;
        return (
            <div className="pure-g">
                <div className="pure-u-2-3" />
                <div className="pure-u-1-3" id="pr-context">
                    <div className="sender"><a href={sender.htmlUrl} title={sender.login}><img src={sender.avatarUrl} alt={sender.login} /></a></div>
                    <div className="github-pr">
                        <div className="number"><a href={pr.htmlUrl}>#{ctx.number}</a></div>
                        <div className="title"><a href={pr.htmlUrl}>{pr.title}</a></div>
                    </div>
                </div>
            </div>
        )
    }
}