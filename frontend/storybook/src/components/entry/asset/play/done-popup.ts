import { argsToAttrs } from "@utils/attributes";
// import "@elements/entry/asset/play/jig/done-popup";

export default {
    title: "Entry / Jig / Play",
};

interface Args {
    score: number;
}

const DEFAULT_ARGS: Args = {
    score: 90,
};

export const DonePopup = (props?: Partial<Args>) => {
    props = props ? { ...DEFAULT_ARGS, ...props } : DEFAULT_ARGS;

    return `
        <jig-play-done-popup ${argsToAttrs(props)}>
            <div slot="actions">
                <jig-play-done-action></jig-play-done-action>
            </div>
        </jig-play-done-popup>
    `;
};
DonePopup.args = DEFAULT_ARGS;
