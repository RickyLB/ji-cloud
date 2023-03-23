import { argsToAttrs } from "@utils/attributes";
// import "@elements/entry/asset/play/jig/time-up-popup";

export default {
    title: "Entry / Jig / Play",
};

interface Args {
    score: number;
}

const DEFAULT_ARGS: Args = {
    score: 90,
};

export const TimeUpPopup = (props?: Partial<Args>) => {
    props = props ? { ...DEFAULT_ARGS, ...props } : DEFAULT_ARGS;

    return `
        <jig-play-time-up-popup ${argsToAttrs(props)}>
            <div slot="actions">
                <jig-play-done-action></jig-play-done-action>
            </div>
        </jig-play-time-up-popup>
    `;
};
TimeUpPopup.args = DEFAULT_ARGS;
