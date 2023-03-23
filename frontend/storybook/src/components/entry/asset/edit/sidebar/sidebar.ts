import { argsToAttrs, argToAttr } from "@utils/attributes";
// import "@elements/entry/asset/edit/sidebar/sidebar";
import "@elements/entry/asset/edit/sidebar/header";
import "@elements/entry/asset/edit/sidebar/filler";
import { mapToString, arrayIndex } from "@utils/array";
import { Module } from "../../_common/sidebar-modules/module";
import { Header } from "./header";
import { Publish } from "./publish";

export default {
    title: "Entry / Jig / Edit / Sidebar",
};

interface Args {
    nModules: number;
    menuIndex: number;
    dragIndex: number;
    collapsed: boolean;
}

const DEFAULT_ARGS_SIDEBAR: Args = {
    nModules: 10,
    menuIndex: 1,
    dragIndex: -1,
    collapsed: false,
};

export const Sidebar = (props?: Partial<Args> & { slot?: string }) => {
    props = props
        ? { ...DEFAULT_ARGS_SIDEBAR, ...props }
        : DEFAULT_ARGS_SIDEBAR;

    const { slot, menuIndex, dragIndex, nModules } = props;

    return `
        <jig-edit-sidebar ${argToAttr(["collapsed", props.collapsed])} ${
        slot && `slot="${slot}"`
    }>
            ${Header({ collapsed: props.collapsed })}
            ${mapToString(arrayIndex(nModules), (index) => {
                return Module({
                    module: index === 0 ? "cover" : "memory",
                    rawIndex: index,
                    menuOpen: index === menuIndex,
                    slot: index === 0 ? "cover-module" : "modules",
                    selected: index === 1,
                    makeDemoRoomAtTop: false,
                    showAdd: index !== nModules - 1,
                    dragging: index === dragIndex,
                    collapsed: props.collapsed,
                });
            })}
            ${Publish({
                collapsed: props.collapsed,
                slot: "modules",
            })}
        </jig-edit-sidebar>
    `;
};
Sidebar.args = DEFAULT_ARGS_SIDEBAR;

interface DragArgs {
    dragX: number;
    dragY: number;
}
const DEFAULT_ARGS_DRAGGING: Args & DragArgs = {
    nModules: 10,
    menuIndex: -1,
    dragIndex: 1,
    dragX: 100,
    dragY: 400,
    collapsed: false,
};

export const Dragging = (
    props?: Partial<Args & DragArgs> & { slot?: string }
) => {
    props = props
        ? { ...DEFAULT_ARGS_DRAGGING, ...props }
        : DEFAULT_ARGS_DRAGGING;

    const { slot, menuIndex, dragIndex, dragX, dragY, nModules } = props;

    return `
        <jig-edit-sidebar ${slot && `slot="${slot}"`} ${
        props.collapsed ? "collapsed" : ""
    }>
        <jig-edit-sidebar-header slot="header"> </jig-edit-sidebar-header>
        ${mapToString(arrayIndex(nModules), (index) => {
            const slot = index === 0 ? "cover-module" : "modules";
            const dragging = index === dragIndex;

            return dragging
                ? `<jig-edit-sidebar-filler slot="${slot}"></jig-edit-sidebar-filler>`
                : Module({
                      module: index === 0 ? "cover" : "memory",
                      rawIndex: index,
                      menuOpen: index === menuIndex,
                      selected: index === 1,
                      slot,
                      makeDemoRoomAtTop: false,
                      showAdd: index !== nModules - 1,
                      dragging,
                  });
        })}
        ${Publish({
            slot: "modules",
        })}
        </jig-edit-sidebar>

        ${
            dragIndex >= 0 && dragIndex < nModules
                ? renderDrag(dragX, dragY, dragIndex)
                : ""
        }
    `;
};
Dragging.args = DEFAULT_ARGS_DRAGGING;

function renderDrag(dragX: number, dragY: number, index: number) {
    return Module({
        module: index === 0 ? "cover" : "memory",
        rawIndex: index,
        dragging: true,
        dragX,
        dragY,
    });
}
