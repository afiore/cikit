import humanizeDuration from 'humanize-duration';

export const showDuration = humanizeDuration.humanizer({
    language: "shortEn",
    units: ["m", "s", "ms"],
    languages: {
        shortEn: {
            m: () => "m",
            s: () => "s",
            ms: () => "ms",
        },
    },
});
