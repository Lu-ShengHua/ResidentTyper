#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::iter::once;
use std::mem::{size_of, zeroed};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr::{null, null_mut};
use std::sync::atomic::{
    AtomicBool, AtomicI32, AtomicIsize, AtomicU32, AtomicU64, AtomicUsize, Ordering,
};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use windows::core::{BSTR, PCWSTR, VARIANT};
use windows::Win32::Foundation::HWND as WindowsHwnd;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationInvokePattern,
    IUIAutomationLegacyIAccessiblePattern, IUIAutomationScrollItemPattern,
    IUIAutomationSelectionItemPattern, IUIAutomationValuePattern, TreeScope_Children,
    TreeScope_Descendants, UIA_ButtonControlTypeId, UIA_CheckBoxControlTypeId,
    UIA_ControlTypePropertyId, UIA_DataItemControlTypeId, UIA_EditControlTypeId,
    UIA_InvokePatternId, UIA_LegacyIAccessiblePatternId, UIA_NamePropertyId, UIA_PaneControlTypeId,
    UIA_ScrollItemPatternId, UIA_SelectionItemPatternId, UIA_TreeItemControlTypeId,
    UIA_ValuePatternId, UIA_WindowControlTypeId,
};

type Hwnd = isize;
type Hinstance = isize;
type Hmenu = isize;
type Hbrush = isize;
type Hcursor = isize;
type Hfont = isize;
type Hhook = isize;
type Hdc = isize;
type Lparam = isize;
type Lresult = isize;
type Wparam = usize;

const APP_CLASS: &str = "ResidentTyperDemoWindow";
const SELECTION_POPUP_CLASS: &str = "ResidentTyperSelectionPopup";
const SETTINGS_WINDOW_CLASS: &str = "ResidentTyperSettingsWindow";
const DOCK_HANDLE_CLASS: &str = "ResidentTyperDockHandle";
const APP_TITLE: &str = "神外小助手";
const APP_VERSION: &str = "v1.3.43";
const APP_UPDATED_AT: &str = "2026-09-04";
const DEFAULT_REMOTE_SIGN_SERVER: &str = "192.168.1.2";
const DEFAULT_REMOTE_SIGN_PORT: u16 = 41888;
const STARTUP_VALUE_NAME: &str = "ResidentTyperAssistant";
const EDITABLE_TEXT_FOLDER: &str = "可修改文本";
const TEMPLATE_TEXT_FOLDER: &str = "病程模板";
const DISEASE_TEMPLATE_FOLDER: &str = "DiseasePlaceHolder";
const DEFAULT_WINDOW_X: i32 = 120;
const DEFAULT_WINDOW_Y: i32 = 80;
const DEFAULT_WINDOW_W: i32 = 326;
const DEFAULT_WINDOW_H: i32 = 890;
const API_WINDOW_W: i32 = 760;
const API_WINDOW_H: i32 = 890;
const COURSE_SPEED_PERCENT: u64 = 35;

const ID_SENIOR_FIRST: i32 = 101;
const ID_POST_FIRST: i32 = 102;
const ID_POST_SENIOR_FIRST: i32 = 103;
const ID_POST_DAILY: i32 = 104;
const ID_PREOP_DISCUSSION: i32 = 105;
const ID_STOP: i32 = 106;
const ID_DISCHARGE_CERT: i32 = 107;
const ID_DISCHARGE_RECORD: i32 = 108;
const ID_PREOP_SUMMARY: i32 = 109;
const ID_FIRST_COURSE: i32 = 110;
const ID_STATUS: i32 = 202;
const ID_SKIP_REMINDER: i32 = 203;
const ID_VERSION: i32 = 204;
const ID_TAB_TEMPLATES: i32 = 210;
const ID_TAB_SHORTCUTS: i32 = 211;
const ID_TAB_API: i32 = 214;
const ID_THEME_TOGGLE: i32 = 212;
const ID_DOCK_TOGGLE: i32 = 213;
const ID_SHORTCUT_SAVE_ORDER: i32 = 401;
const ID_SHORTCUT_VT: i32 = 402;
const ID_SHORTCUT_MEDICAL_LAUNCH: i32 = 403;
const ID_SHORTCUT_CREATE_ALL: i32 = 404;
const ID_LOGIN_USER: i32 = 405;
const ID_LOGIN_PASS: i32 = 406;
const ID_SHORTCUT_ORDER_LAUNCH: i32 = 407;
const ID_ACCOUNT_QUERY: i32 = 408;
const ID_NURSING_LAUNCH: i32 = 409;
const ID_WARD_BUILDING: i32 = 410;
const ID_WARD_FLOOR: i32 = 411;
const ID_SHORTCUT_CLINICAL_CONTINUE: i32 = 413;
const ID_CLINICAL_LOOP_COUNT: i32 = 414;
const ID_STARTUP_CHECK: i32 = 415;
const ID_CREATE_ALL_BASE_TIME: i32 = 416;
const ID_ATTENDING_SUPERIOR: i32 = 417;
const ID_CHIEF_SUPERIOR: i32 = 418;
const ID_SUPERIOR_LIST: i32 = 419;
const ID_CLIPBOARD_AUTO: i32 = 420;
const ID_SHORTCUT_CREATE_POSTOP: i32 = 421;
const ID_POSTOP_BASE_TIME: i32 = 422;
const ID_ACCOUNT_DIALOG_CLOSE: i32 = 423;
const ID_SHORTCUT_REMOTE_SIGN: i32 = 424;
const ID_CLEANUP_RUNNING_APPS: i32 = 425;
const ID_HTN: i32 = 301;
const ID_DM: i32 = 302;
const ID_FAST_HR: i32 = 303;
const ID_SEVERE: i32 = 304;
const ID_DISEASE_TOGGLE: i32 = 501;
const ID_DISEASE_LIST: i32 = 502;
const ID_TRAY_SHOW: i32 = 9001;
const ID_TRAY_TASKBAR: i32 = 9002;
const ID_TRAY_DOCK: i32 = 9003;
const ID_TRAY_DAY: i32 = 9004;
const ID_TRAY_NIGHT: i32 = 9005;
const ID_TRAY_STARTUP: i32 = 9006;
const ID_TRAY_EXIT: i32 = 9007;
const ID_TRAY_SETTINGS: i32 = 9008;
const ID_SETTINGS_MEDICAL_PATH: i32 = 9101;
const ID_SETTINGS_ORDER_PATH: i32 = 9102;
const ID_SETTINGS_NURSING_PATH: i32 = 9113;
const ID_SETTINGS_BROWSE_MEDICAL: i32 = 9103;
const ID_SETTINGS_BROWSE_ORDER: i32 = 9104;
const ID_SETTINGS_BROWSE_NURSING: i32 = 9114;
const ID_SETTINGS_SAVE: i32 = 9105;
const ID_SETTINGS_CANCEL: i32 = 9106;
const ID_SETTINGS_DOCK_HIDE_DELAY: i32 = 9107;
const ID_SETTINGS_ALT_F4: i32 = 9108;
const ID_SETTINGS_ALT_F4_NO: i32 = 9115;
const ID_SETTINGS_OPEN_MAIN: i32 = 9112;
const ID_SETTINGS_RES_1080: i32 = 9130;
const ID_SETTINGS_RES_2160: i32 = 9131;
const ID_SETTINGS_API_MODE: i32 = 9132;
const ID_SETTINGS_REMOTE_SIGN_SERVER: i32 = 9133;
const ID_SETTINGS_REMOTE_SIGN_PORT: i32 = 9134;
const ID_SETTINGS_AUTO_REMOTE_SIGN: i32 = 9135;
const ID_API_PATIENT: i32 = 9201;
const ID_API_PATIENT_NEW: i32 = 9202;
const ID_API_PATIENT_ADD: i32 = 9203;
const ID_API_PATIENT_DELETE: i32 = 9204;
const ID_API_DOC_TYPE: i32 = 9205;
const ID_API_DESCRIPTION: i32 = 9206;
const ID_API_TEMPLATE_PROMPT: i32 = 9207;
const ID_API_SEND: i32 = 9208;
const ID_API_TYPE_RESULT: i32 = 9210;
const ID_API_DOC_BASE: i32 = 9220;
const ID_API_CONTEXT_WRITEBACK: i32 = 9299;

const API_DOC_NAMES: [&str; 8] = [
    "首程",
    "首次查房",
    "术后首程",
    "术后查房",
    "术后日常",
    "术前讨论",
    "出院诊断",
    "出院记录",
];
const API_DOC_COUNT: usize = API_DOC_NAMES.len();

const DISEASE_ITEMS: [&str; 17] = [
    "通用",
    "颅内占位性病变",
    "烟雾病",
    "大脑中动脉狭窄",
    "椎动脉狭窄",
    "颈动脉狭窄(介入)",
    "颈动脉狭窄(CEA)",
    "基底动脉狭窄",
    "慢性膜下血肿",
    "脑积水",
    "颅内动脉瘤(介入)",
    "颅内动脉瘤(开颅)",
    "颈椎病",
    "腰椎管狭窄",
    "椎管内占位",
    "颅脑外伤",
    "垂体瘤",
];
const WARD_BUILDINGS: [&str; 2] = ["C", "D"];
const ACCOUNT_QUERY_ROWS: [(&str, &str); 14] = [
    ("张东", "5064"),
    ("刘加春", "3656"),
    ("裴傲", "3097"),
    ("祁鹏", "3924"),
    ("谢红雯", "2555"),
    ("胡深", "4451"),
    ("杨希孟", "4600"),
    ("王海峰", "4774"),
    ("陈鲲鹏", "4709"),
    ("张顺", "5081"),
    ("张绍森", "5136"),
    ("王乔", "5137"),
    ("黄亮然", "5274"),
    ("卢盛华", "5275"),
];

const BUTTONS: [ButtonDef; 9] = [
    ButtonDef {
        id: ID_FIRST_COURSE,
        action: ButtonAction::Flow(FlowKind::FirstCourse),
    },
    ButtonDef {
        id: ID_SENIOR_FIRST,
        action: ButtonAction::Flow(FlowKind::SeniorFirst),
    },
    ButtonDef {
        id: ID_PREOP_SUMMARY,
        action: ButtonAction::Template("preop_summary.txt"),
    },
    ButtonDef {
        id: ID_PREOP_DISCUSSION,
        action: ButtonAction::Flow(FlowKind::PreopDiscussion),
    },
    ButtonDef {
        id: ID_POST_FIRST,
        action: ButtonAction::Template("post_op_first.txt"),
    },
    ButtonDef {
        id: ID_POST_SENIOR_FIRST,
        action: ButtonAction::Flow(FlowKind::PostSeniorFirst),
    },
    ButtonDef {
        id: ID_POST_DAILY,
        action: ButtonAction::Flow(FlowKind::PostDaily),
    },
    ButtonDef {
        id: ID_DISCHARGE_CERT,
        action: ButtonAction::Flow(FlowKind::DischargeCertificate),
    },
    ButtonDef {
        id: ID_DISCHARGE_RECORD,
        action: ButtonAction::Template("discharge_record.txt"),
    },
];

static STOP_TYPING: AtomicBool = AtomicBool::new(false);
static RIGHT_CTRL_DOWN: AtomicBool = AtomicBool::new(false);
static RIGHT_ALT_DOWN: AtomicBool = AtomicBool::new(false);
static INSERT_HOTKEY_DOWN: AtomicBool = AtomicBool::new(false);
static ALT_F1_HOTKEY_DOWN: AtomicBool = AtomicBool::new(false);
static ALT_F2_HOTKEY_DOWN: AtomicBool = AtomicBool::new(false);
static CTRL_DOWN: AtomicBool = AtomicBool::new(false);
static ALT_DOWN: AtomicBool = AtomicBool::new(false);
static CLIPBOARD_HOTKEY_DOWN: AtomicBool = AtomicBool::new(false);
// Keep the window that owned the keyboard focus before Ctrl+Alt+V was consumed.
// The assistant is topmost, so restoring this window is necessary before typing.
static CLIPBOARD_TARGET_WINDOW: AtomicIsize = AtomicIsize::new(0);
static CLIPBOARD_TYPING_RUNNING: AtomicBool = AtomicBool::new(false);
static DISEASE_DROPDOWN_OPEN: AtomicBool = AtomicBool::new(false);
static SUPERIOR_PANEL_OPEN: AtomicBool = AtomicBool::new(false);
static SUPERIOR_TARGET: AtomicI32 = AtomicI32::new(0);
static TEMPLATE_PAGE_VISIBLE: AtomicBool = AtomicBool::new(true);
static API_PAGE_VISIBLE: AtomicBool = AtomicBool::new(false);
static CHECKBOX_STATES: AtomicU32 = AtomicU32::new(0);
static DARK_THEME: AtomicBool = AtomicBool::new(false);
static OPEN_PROGRAM_RUNNING: AtomicBool = AtomicBool::new(false);
static PENDING_MEDICAL_LAUNCH: AtomicBool = AtomicBool::new(false);
static PENDING_ORDER_LAUNCH: AtomicBool = AtomicBool::new(false);
static REMOTE_SIGN_RUNNING: AtomicBool = AtomicBool::new(false);
static CLEANUP_RUNNING: AtomicBool = AtomicBool::new(false);
static SAVE_ORDER_RUNNING: AtomicBool = AtomicBool::new(false);
static CLINICAL_PATH_RUNNING: AtomicBool = AtomicBool::new(false);
static CREATE_ALL_COURSES_RUNNING: AtomicBool = AtomicBool::new(false);
static CURRENT_COURSE_ITEM: AtomicUsize = AtomicUsize::new(0);
static COURSE_RUN_ID: AtomicU64 = AtomicU64::new(0);
static VTE_ALERT_SHOWN: AtomicBool = AtomicBool::new(false);
static DOCK_VISIBLE: AtomicBool = AtomicBool::new(true);
static DOCK_TARGET_VISIBLE: AtomicBool = AtomicBool::new(true);
static DOCK_ENABLED: AtomicBool = AtomicBool::new(true);
static DOCK_SUSPEND_UNTIL_MS: AtomicU64 = AtomicU64::new(0);
static DOCK_HIDE_DELAY_MS: AtomicU64 = AtomicU64::new(0);
static DOCK_HIDE_DEADLINE_MS: AtomicU64 = AtomicU64::new(0);
static DOCK_EDGE: AtomicI32 = AtomicI32::new(DOCK_EDGE_RIGHT);
static TASKBAR_VISIBLE: AtomicBool = AtomicBool::new(true);
static TRAY_ICON_ADDED: AtomicBool = AtomicBool::new(false);
static DOCK_REGION_HIDDEN: AtomicBool = AtomicBool::new(false);
static DOCK_CHROMELESS_HIDDEN: AtomicBool = AtomicBool::new(false);
static DOCK_LAST_SCREEN_W: AtomicI32 = AtomicI32::new(0);
static DOCK_LAST_SCREEN_H: AtomicI32 = AtomicI32::new(0);
static mut DOCK_HANDLE_HWND: Hwnd = 0;
static LAUNCH_RESOLUTION_PROFILE: AtomicUsize = AtomicUsize::new(0);
static CLINICAL_PATH_RESOLUTION_PROFILE: AtomicUsize = AtomicUsize::new(0);
static SETTINGS_API_MODE_CHECKED: AtomicBool = AtomicBool::new(false);
static API_DOC_GENERATED_MASK: AtomicU32 = AtomicU32::new(0);
static API_CONTEXT_DOC_INDEX: AtomicI32 = AtomicI32::new(-1);
static mut KEYBOARD_HOOK: Hhook = 0;
static mut MAIN_FONT: Hfont = 0;
static mut TITLE_FONT: Hfont = 0;
static mut BUTTON_FONT: Hfont = 0;
static mut NOTE_FONT: Hfont = 0;
static mut DAY_BRUSH: Hbrush = 0;
static mut NIGHT_BRUSH: Hbrush = 0;
static mut DAY_CONTROL_BRUSH: Hbrush = 0;
static mut NIGHT_CONTROL_BRUSH: Hbrush = 0;
static mut DAY_POPUP_BRUSH: Hbrush = 0;
static mut NIGHT_POPUP_BRUSH: Hbrush = 0;
static mut TEMPLATE_CONTROLS: [Hwnd; 64] = [0; 64];
static mut TEMPLATE_CONTROL_COUNT: usize = 0;
static mut SHORTCUT_CONTROLS: [Hwnd; 32] = [0; 32];
static mut SHORTCUT_CONTROL_COUNT: usize = 0;
static mut API_CONTROLS: [Hwnd; 32] = [0; 32];
static mut API_CONTROL_COUNT: usize = 0;
const TEMPLATE_CONTROL_LIMIT: usize = 64;
const SHORTCUT_CONTROL_LIMIT: usize = 32;
const API_CONTROL_LIMIT: usize = 32;
static mut APP: AppState = AppState {
    hwnd: 0,
    logo: 0,
    header: 0,
    divider: 0,
    status: 0,
    version: 0,
    tab_templates: 0,
    tab_shortcuts: 0,
    tab_api: 0,
    settings_button: 0,
    cleanup_button: 0,
    theme_toggle: 0,
    dock_toggle: 0,
    disease_toggle: 0,
    disease_popup: 0,
    disease_list: 0,
    skip_reminder: 0,
    login_user: 0,
    login_pass: 0,
    account_popup: 0,
    clinical_loop_count: 0,
    create_all_base_time: 0,
    postop_base_time: 0,
    attending_superior: 0,
    chief_superior: 0,
    superior_header: 0,
    superior_popup: 0,
    superior_list: 0,
    save_order_hotkey: 0,
    startup_check: 0,
    clipboard_auto: 0,
    ward_building: 0,
    ward_floor: 0,
    api_patient: 0,
    api_patient_new: 0,
    api_doc_type: 0,
    api_description: 0,
    api_template_prompt: 0,
    api_send: 0,
    api_type_result: 0,
    api_doc_rows: [0; API_DOC_COUNT],
    htn: 0,
    dm: 0,
    fast_hr: 0,
    severe: 0,
};

#[derive(Clone, Copy)]
struct SettingsDialogState {
    hwnd: Hwnd,
    medical_path: Hwnd,
    order_path: Hwnd,
    nursing_path: Hwnd,
    remote_sign_server: Hwnd,
    remote_sign_port: Hwnd,
    auto_remote_sign: Hwnd,
    dock_hide_delay: Hwnd,
    clinical_res_1080: Hwnd,
    clinical_res_2160: Hwnd,
    launch_alt_f4: Hwnd,
    launch_alt_f4_no: Hwnd,
    startup_check: Hwnd,
    api_mode: Hwnd,
}

static mut SETTINGS_DIALOG: SettingsDialogState = SettingsDialogState {
    hwnd: 0,
    medical_path: 0,
    order_path: 0,
    nursing_path: 0,
    remote_sign_server: 0,
    remote_sign_port: 0,
    auto_remote_sign: 0,
    dock_hide_delay: 0,
    clinical_res_1080: 0,
    clinical_res_2160: 0,
    launch_alt_f4: 0,
    launch_alt_f4_no: 0,
    startup_check: 0,
    api_mode: 0,
};

#[derive(Clone, Copy)]
struct ButtonDef {
    id: i32,
    action: ButtonAction,
}

#[derive(Clone, Copy)]
enum ButtonAction {
    Flow(FlowKind),
    Template(&'static str),
}

#[derive(Clone, Copy)]
enum FlowKind {
    FirstCourse,
    SeniorFirst,
    PreopDiscussion,
    PostSeniorFirst,
    PostDaily,
    DischargeCertificate,
}

#[derive(Clone, Copy)]
struct AppState {
    hwnd: Hwnd,
    logo: Hwnd,
    header: Hwnd,
    divider: Hwnd,
    status: Hwnd,
    version: Hwnd,
    tab_templates: Hwnd,
    tab_shortcuts: Hwnd,
    tab_api: Hwnd,
    settings_button: Hwnd,
    cleanup_button: Hwnd,
    theme_toggle: Hwnd,
    dock_toggle: Hwnd,
    disease_toggle: Hwnd,
    disease_popup: Hwnd,
    disease_list: Hwnd,
    skip_reminder: Hwnd,
    login_user: Hwnd,
    login_pass: Hwnd,
    account_popup: Hwnd,
    clinical_loop_count: Hwnd,
    create_all_base_time: Hwnd,
    postop_base_time: Hwnd,
    attending_superior: Hwnd,
    chief_superior: Hwnd,
    superior_header: Hwnd,
    superior_popup: Hwnd,
    superior_list: Hwnd,
    save_order_hotkey: Hwnd,
    startup_check: Hwnd,
    clipboard_auto: Hwnd,
    ward_building: Hwnd,
    ward_floor: Hwnd,
    api_patient: Hwnd,
    api_patient_new: Hwnd,
    api_doc_type: Hwnd,
    api_description: Hwnd,
    api_template_prompt: Hwnd,
    api_send: Hwnd,
    api_type_result: Hwnd,
    api_doc_rows: [Hwnd; API_DOC_COUNT],
    htn: Hwnd,
    dm: Hwnd,
    fast_hr: Hwnd,
    severe: Hwnd,
}

#[derive(Default, Clone, Copy)]
struct Options {
    hypertension: bool,
    diabetes: bool,
    fast_hr: bool,
    severe: bool,
}

#[derive(Clone, Copy)]
struct Vitals {
    sbp: u32,
    dbp: u32,
    hr: u32,
    rr: u32,
    temp10: u32,
}

#[derive(Default, Clone)]
struct Settings {
    username: String,
    password: String,
    skip_reminder: bool,
    window_x: i32,
    window_y: i32,
    window_w: i32,
    window_h: i32,
    disease_index: usize,
    dark_theme: bool,
    dock_enabled: bool,
    taskbar_visible: bool,
    startup_enabled: bool,
    clinical_loop_count: String,
    create_all_base_time: String,
    attending_superior: String,
    chief_superior: String,
    clipboard_auto: bool,
    save_order_hotkey: bool,
    postop_base_time: String,
    ward_building_index: usize,
    ward_floor: u32,
    medical_system_path: String,
    order_system_path: String,
    nursing_system_path: String,
    remote_sign_server: String,
    remote_sign_port: u16,
    auto_remote_sign: bool,
    dock_hide_delay_ms: u64,
    launch_resolution_profile: usize,
    clinical_path_resolution_profile: usize,
    launch_alt_f4: bool,
    api_mode_enabled: bool,
    deepseek_api_key: String,
    api_prompt_text: String,
}

#[derive(Clone, Copy)]
struct LaunchPoints {
    medical_focus: (i32, i32),
    medical_login: (i32, i32),
}

#[derive(Clone, Copy)]
struct SimpleDate {
    year: i32,
    month: u32,
    day: u32,
}

#[derive(Clone, Copy)]
struct CourseCreateSpec {
    parent: &'static str,
    template: &'static str,
    day_offset: u32,
    clock: &'static str,
    superior_kind: Option<SuperiorKind>,
    document_title: CourseDocumentTitle,
}

#[derive(Clone, Copy)]
enum SuperiorKind {
    Attending,
    Chief,
}

#[derive(Clone, Copy)]
enum CourseDocumentTitle {
    Default,
    PostopDayOneSurgeon,
    Fixed(&'static str),
}

#[derive(Clone, Copy)]
enum CourseBatchKind {
    Preop,
    Postop,
}

const PREOP_COURSE_SPECS: [CourseCreateSpec; 9] = [
    CourseCreateSpec {
        parent: "入院记录",
        template: "神经外科入院记录二",
        day_offset: 0,
        clock: "21:00:00",
        superior_kind: None,
        document_title: CourseDocumentTitle::Default,
    },
    CourseCreateSpec {
        parent: "病程记录",
        template: "首次病程记录（神经外科）",
        day_offset: 0,
        clock: "21:00:00",
        superior_kind: None,
        document_title: CourseDocumentTitle::Default,
    },
    CourseCreateSpec {
        parent: "病程记录",
        template: "首次上级医师查房记录",
        day_offset: 1,
        clock: "08:00:00",
        superior_kind: Some(SuperiorKind::Attending),
        document_title: CourseDocumentTitle::Default,
    },
    CourseCreateSpec {
        parent: "病程记录",
        template: "首次上级医师查房记录",
        day_offset: 2,
        clock: "08:00:00",
        superior_kind: Some(SuperiorKind::Chief),
        document_title: CourseDocumentTitle::Default,
    },
    CourseCreateSpec {
        parent: "病程记录",
        template: "日常病程记录",
        day_offset: 3,
        clock: "08:00:00",
        superior_kind: None,
        document_title: CourseDocumentTitle::Default,
    },
    CourseCreateSpec {
        parent: "讨论记录",
        template: "术前讨论（通用）",
        day_offset: 1,
        clock: "08:00:00",
        superior_kind: None,
        document_title: CourseDocumentTitle::Default,
    },
    CourseCreateSpec {
        parent: "讨论记录",
        template: "术前小结",
        day_offset: 1,
        clock: "08:00:00",
        superior_kind: None,
        document_title: CourseDocumentTitle::Default,
    },
    CourseCreateSpec {
        parent: "出院记录",
        template: "出院记录-新（神外）",
        day_offset: 5,
        clock: "08:00:00",
        superior_kind: None,
        document_title: CourseDocumentTitle::Default,
    },
    CourseCreateSpec {
        parent: "诊断证明",
        template: "诊断证明（外科）",
        day_offset: 5,
        clock: "08:00:00",
        superior_kind: None,
        document_title: CourseDocumentTitle::Default,
    },
];

const POSTOP_COURSE_SPECS: [CourseCreateSpec; 4] = [
    CourseCreateSpec {
        parent: "病程记录",
        template: "术后首次病程记录",
        day_offset: 0,
        clock: "20:00:00",
        superior_kind: None,
        document_title: CourseDocumentTitle::Default,
    },
    CourseCreateSpec {
        parent: "病程记录",
        template: "术后第1日术者查房记录",
        day_offset: 1,
        clock: "08:00:00",
        superior_kind: None,
        document_title: CourseDocumentTitle::PostopDayOneSurgeon,
    },
    CourseCreateSpec {
        parent: "病程记录",
        template: "术后第23日查房记录",
        day_offset: 2,
        clock: "08:00:00",
        superior_kind: None,
        document_title: CourseDocumentTitle::Fixed("术后第2日查房记录"),
    },
    CourseCreateSpec {
        parent: "病程记录",
        template: "术后第23日查房记录",
        day_offset: 3,
        clock: "08:00:00",
        superior_kind: None,
        document_title: CourseDocumentTitle::Fixed("术后第3日查房记录"),
    },
];

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
struct DrawItemStruct {
    ctl_type: u32,
    ctl_id: u32,
    item_id: u32,
    item_action: u32,
    item_state: u32,
    hwnd_item: Hwnd,
    hdc: Hdc,
    rc_item: Rect,
    item_data: usize,
}

#[repr(C)]
struct WndClassW {
    style: u32,
    lpfn_wnd_proc: extern "system" fn(Hwnd, u32, Wparam, Lparam) -> Lresult,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: Hinstance,
    h_icon: isize,
    h_cursor: Hcursor,
    hbr_background: Hbrush,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
}

#[repr(C)]
struct Msg {
    hwnd: Hwnd,
    message: u32,
    w_param: Wparam,
    l_param: Lparam,
    time: u32,
    pt_x: i32,
    pt_y: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct KeybdInput {
    w_vk: u16,
    w_scan: u16,
    dw_flags: u32,
    time: u32,
    dw_extra_info: usize,
}

#[repr(C)]
union InputUnion {
    mi: MouseInput,
    ki: KeybdInput,
    _padding: [usize; 4],
}

#[repr(C)]
struct Input {
    input_type: u32,
    u: InputUnion,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct MouseInput {
    dx: i32,
    dy: i32,
    mouse_data: u32,
    dw_flags: u32,
    time: u32,
    dw_extra_info: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct KbdllHookStruct {
    vk_code: u32,
    scan_code: u32,
    flags: u32,
    time: u32,
    dw_extra_info: usize,
}

#[repr(C)]
struct InitCommonControlsEx {
    size: u32,
    classes: u32,
}

#[repr(C)]
#[derive(Default)]
struct WinSystemTime {
    year: u16,
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    milliseconds: u16,
}

#[repr(C)]
struct NotifyIconDataW {
    cb_size: u32,
    hwnd: Hwnd,
    id: u32,
    flags: u32,
    callback_message: u32,
    icon: isize,
    tip: [u16; 128],
    state: u32,
    state_mask: u32,
    info: [u16; 256],
    timeout_or_version: u32,
    info_title: [u16; 64],
    info_flags: u32,
    guid_item: [u8; 16],
    balloon_icon: isize,
}

#[repr(C)]
struct OpenFileNameW {
    l_struct_size: u32,
    hwnd_owner: Hwnd,
    h_instance: Hinstance,
    lpstr_filter: *const u16,
    lpstr_custom_filter: *mut u16,
    n_max_cust_filter: u32,
    n_filter_index: u32,
    lpstr_file: *mut u16,
    n_max_file: u32,
    lpstr_file_title: *mut u16,
    n_max_file_title: u32,
    lpstr_initial_dir: *const u16,
    lpstr_title: *const u16,
    flags: u32,
    n_file_offset: u16,
    n_file_extension: u16,
    lpstr_def_ext: *const u16,
    l_cust_data: Lparam,
    lpfn_hook: *mut std::ffi::c_void,
    lp_template_name: *const u16,
    pv_reserved: *mut std::ffi::c_void,
    dw_reserved: u32,
    flags_ex: u32,
}

#[link(name = "user32")]
extern "system" {
    fn AppendMenuW(menu: Hmenu, flags: u32, item: usize, text: *const u16) -> i32;
    fn AttachThreadInput(id_attach: u32, id_attach_to: u32, attach: i32) -> i32;
    fn BringWindowToTop(hwnd: Hwnd) -> i32;
    fn CallNextHookEx(hhk: Hhook, n_code: i32, w_param: Wparam, l_param: Lparam) -> Lresult;
    fn CheckMenuItem(menu: Hmenu, item: u32, check: u32) -> u32;
    fn CreatePopupMenu() -> Hmenu;
    fn RegisterClassW(lp_wnd_class: *const WndClassW) -> u16;
    fn CreateWindowExW(
        dw_ex_style: u32,
        lp_class_name: *const u16,
        lp_window_name: *const u16,
        dw_style: u32,
        x: i32,
        y: i32,
        n_width: i32,
        n_height: i32,
        h_wnd_parent: Hwnd,
        h_menu: Hmenu,
        h_instance: Hinstance,
        lp_param: *mut std::ffi::c_void,
    ) -> Hwnd;
    fn DefWindowProcW(hwnd: Hwnd, msg: u32, w_param: Wparam, l_param: Lparam) -> Lresult;
    fn DispatchMessageW(lp_msg: *const Msg) -> Lresult;
    fn DestroyMenu(menu: Hmenu) -> i32;
    fn FindWindowW(lp_class_name: *const u16, lp_window_name: *const u16) -> Hwnd;
    fn GetMessageW(lp_msg: *mut Msg, hwnd: Hwnd, msg_filter_min: u32, msg_filter_max: u32) -> i32;
    fn GetModuleHandleW(lp_module_name: *const u16) -> Hinstance;
    fn GetClientRect(hwnd: Hwnd, lp_rect: *mut Rect) -> i32;
    fn GetCursorPos(lp_point: *mut Point) -> i32;
    fn GetForegroundWindow() -> Hwnd;
    fn GetSystemMetrics(n_index: i32) -> i32;
    fn GetWindowRect(hwnd: Hwnd, lp_rect: *mut Rect) -> i32;
    fn GetWindowLongPtrW(hwnd: Hwnd, index: i32) -> isize;
    fn GetWindowThreadProcessId(hwnd: Hwnd, process_id: *mut u32) -> u32;
    fn InvalidateRect(hwnd: Hwnd, lp_rect: *const Rect, b_erase: i32) -> i32;
    fn IsWindow(hwnd: Hwnd) -> i32;
    fn IsWindowVisible(hwnd: Hwnd) -> i32;
    fn LoadCursorW(h_instance: Hinstance, lp_cursor_name: *const u16) -> Hcursor;
    fn LoadImageW(
        h_instance: Hinstance,
        name: *const u16,
        image_type: u32,
        cx: i32,
        cy: i32,
        flags: u32,
    ) -> isize;
    fn MessageBoxW(hwnd: Hwnd, text: *const u16, caption: *const u16, typ: u32) -> i32;
    fn MapVirtualKeyW(u_code: u32, u_map_type: u32) -> u32;
    fn OpenClipboard(hwnd_new_owner: Hwnd) -> i32;
    fn CloseClipboard() -> i32;
    fn GetClipboardData(format: u32) -> isize;
    fn IsClipboardFormatAvailable(format: u32) -> i32;
    fn MoveWindow(hwnd: Hwnd, x: i32, y: i32, n_width: i32, n_height: i32, repaint: i32) -> i32;
    fn PostMessageW(hwnd: Hwnd, msg: u32, w_param: Wparam, l_param: Lparam) -> i32;
    fn PostQuitMessage(exit_code: i32);
    fn SendInput(c_inputs: u32, p_inputs: *const Input, cb_size: i32) -> u32;
    fn SendMessageW(hwnd: Hwnd, msg: u32, w_param: Wparam, l_param: Lparam) -> Lresult;
    fn SetActiveWindow(hwnd: Hwnd) -> Hwnd;
    fn SetCursorPos(x: i32, y: i32) -> i32;
    fn SetForegroundWindow(hwnd: Hwnd) -> i32;
    fn SetFocus(hwnd: Hwnd) -> Hwnd;
    fn SetClassLongPtrW(hwnd: Hwnd, n_index: i32, dw_new_long: isize) -> isize;
    fn SetWindowPos(
        hwnd: Hwnd,
        hwnd_insert_after: Hwnd,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        flags: u32,
    ) -> i32;
    fn SetWindowRgn(hwnd: Hwnd, region: isize, redraw: i32) -> i32;
    fn SetTimer(
        hwnd: Hwnd,
        id_event: usize,
        elapse: u32,
        timer_func: *mut std::ffi::c_void,
    ) -> usize;
    fn SetWindowsHookExW(
        id_hook: i32,
        lpfn: extern "system" fn(i32, Wparam, Lparam) -> Lresult,
        hmod: Hinstance,
        dw_thread_id: u32,
    ) -> Hhook;
    fn SetWindowTextW(hwnd: Hwnd, text: *const u16) -> i32;
    fn SetWindowLongPtrW(hwnd: Hwnd, index: i32, value: isize) -> isize;
    fn ShowWindow(hwnd: Hwnd, cmd_show: i32) -> i32;
    fn TrackPopupMenu(
        menu: Hmenu,
        flags: u32,
        x: i32,
        y: i32,
        reserved: i32,
        hwnd: Hwnd,
        rect: *const Rect,
    ) -> i32;
    fn TranslateMessage(lp_msg: *const Msg) -> i32;
    fn KillTimer(hwnd: Hwnd, id_event: usize) -> i32;
    fn UnhookWindowsHookEx(hhk: Hhook) -> i32;
    fn UpdateWindow(hwnd: Hwnd) -> i32;
}

#[link(name = "shell32")]
extern "system" {
    fn Shell_NotifyIconW(message: u32, data: *mut NotifyIconDataW) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentThreadId() -> u32;
    fn GlobalLock(memory: isize) -> *const u16;
    fn GlobalUnlock(memory: isize) -> i32;
}

#[link(name = "comctl32")]
extern "system" {
    fn InitCommonControlsEx(init: *const InitCommonControlsEx) -> i32;
}

#[link(name = "comdlg32")]
extern "system" {
    fn GetOpenFileNameW(open_file_name: *mut OpenFileNameW) -> i32;
}

#[link(name = "gdi32")]
extern "system" {
    fn CreateSolidBrush(color: u32) -> Hbrush;
    fn CreatePen(style: i32, width: i32, color: u32) -> isize;
    fn CreateFontW(
        c_height: i32,
        c_width: i32,
        c_escapement: i32,
        c_orientation: i32,
        c_weight: i32,
        b_italic: u32,
        b_underline: u32,
        b_strike_out: u32,
        i_char_set: u32,
        i_out_precision: u32,
        i_clip_precision: u32,
        i_quality: u32,
        i_pitch_and_family: u32,
        psz_face_name: *const u16,
    ) -> Hfont;
    fn SetBkColor(hdc: Hdc, color: u32) -> u32;
    fn DeleteObject(ho: isize) -> i32;
    fn DrawTextW(
        hdc: Hdc,
        lpch_text: *const u16,
        cch_text: i32,
        lprc: *mut Rect,
        format: u32,
    ) -> i32;
    fn FillRect(hdc: Hdc, lprc: *const Rect, hbr: Hbrush) -> i32;
    fn GetStockObject(i: i32) -> isize;
    fn LineTo(hdc: Hdc, x: i32, y: i32) -> i32;
    fn MoveToEx(hdc: Hdc, x: i32, y: i32, old_point: *mut Point) -> i32;
    fn RoundRect(
        hdc: Hdc,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        width: i32,
        height: i32,
    ) -> i32;
    fn CreateRoundRectRgn(
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        width: i32,
        height: i32,
    ) -> isize;
    fn SelectObject(hdc: Hdc, object: isize) -> isize;
    fn BeginPaint(hwnd: Hwnd, paint: *mut PaintStruct) -> Hdc;
    fn EndPaint(hwnd: Hwnd, paint: *const PaintStruct) -> i32;
    fn SetBkMode(hdc: Hdc, mode: i32) -> i32;
    fn SetTextColor(hdc: Hdc, color: u32) -> u32;
}

#[link(name = "dwmapi")]
extern "system" {
    fn DwmSetWindowAttribute(
        hwnd: Hwnd,
        attribute: u32,
        value: *const std::ffi::c_void,
        size: u32,
    ) -> i32;
}

const WS_EX_TOPMOST: u32 = 0x00000008;
const WS_EX_TOOLWINDOW: u32 = 0x00000080;
const WS_EX_STATICEDGE: u32 = 0x00020000;
const WS_EX_APPWINDOW: u32 = 0x00040000;
const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
const WS_POPUP: u32 = 0x80000000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_CHILD: u32 = 0x40000000;
const WS_TABSTOP: u32 = 0x00010000;
const WS_GROUP: u32 = 0x00020000;
const WS_VSCROLL: u32 = 0x00200000;
const BS_OWNERDRAW: u32 = 0x0000000B;
const SS_ICON: u32 = 0x00000003;
const SS_OWNERDRAW: u32 = 0x0000000D;
const LBS_NOTIFY: u32 = 0x00000001;
const LBS_NOINTEGRALHEIGHT: u32 = 0x00000100;
const ES_LEFT: u32 = 0x00000000;
const ES_MULTILINE: u32 = 0x00000004;
const ES_AUTOVSCROLL: u32 = 0x0040;
const ES_WANTRETURN: u32 = 0x1000;
const ICC_DATE_CLASSES: u32 = 0x00000100;
const GDT_VALID: Wparam = 0;

const WM_CREATE: u32 = 0x0001;
const WM_DESTROY: u32 = 0x0002;
const WM_CLOSE: u32 = 0x0010;
const WM_DRAWITEM: u32 = 0x002B;
const WM_SIZE: u32 = 0x0005;
const WM_DISPLAYCHANGE: u32 = 0x007E;
const WM_DPICHANGED: u32 = 0x02E0;
const WM_TIMER: u32 = 0x0113;
const WM_CTLCOLORMSGBOX: u32 = 0x0132;
const WM_CTLCOLOREDIT: u32 = 0x0133;
const WM_CTLCOLORLISTBOX: u32 = 0x0134;
const WM_CTLCOLORBTN: u32 = 0x0135;
const WM_CTLCOLORSTATIC: u32 = 0x0138;
const WM_COMMAND: u32 = 0x0111;
const WM_CONTEXTMENU: u32 = 0x007B;
const WM_PAINT: u32 = 0x000F;
const WM_LBUTTONDBLCLK: u32 = 0x0203;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_MOUSEMOVE: u32 = 0x0200;
const WM_RBUTTONUP: u32 = 0x0205;
const WM_TRAY_ICON: u32 = 0x8001;
const WM_GETTEXT: u32 = 0x000D;
const WM_GETTEXTLENGTH: u32 = 0x000E;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_SETFONT: u32 = 0x0030;
const EM_SETMARGINS: u32 = 0x00D3;
const EC_LEFTMARGIN: u32 = 0x0001;
const EC_RIGHTMARGIN: u32 = 0x0002;
const WM_SETICON: u32 = 0x0080;
const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const WM_SYSKEYDOWN: u32 = 0x0104;
const WM_SYSKEYUP: u32 = 0x0105;
const BM_GETCHECK: u32 = 0x00F0;
const BM_SETCHECK: u32 = 0x00F1;
const BM_CLICK: u32 = 0x00F5;
const STM_SETICON: u32 = 0x0170;
const DTM_GETSYSTEMTIME: u32 = 0x1001;
const DTM_SETSYSTEMTIME: u32 = 0x1002;
const DTM_SETFORMATW: u32 = 0x1032;
const LB_ADDSTRING: u32 = 0x0180;
const LB_SETCURSEL: u32 = 0x0186;
const LB_GETCURSEL: u32 = 0x0188;
const LB_GETTEXT: u32 = 0x0189;
const LB_GETTEXTLEN: u32 = 0x018A;
const LB_FINDSTRINGEXACT: u32 = 0x01A2;
const LBN_SELCHANGE: u16 = 1;
const BST_CHECKED: Lresult = 1;
const SIZE_MINIMIZED: Wparam = 1;
const WH_KEYBOARD_LL: i32 = 13;
const HC_ACTION: i32 = 0;
const INPUT_MOUSE: u32 = 0;
const INPUT_KEYBOARD: u32 = 1;
const KEYEVENTF_EXTENDEDKEY: u32 = 0x0001;
const KEYEVENTF_KEYUP: u32 = 0x0002;
const KEYEVENTF_UNICODE: u32 = 0x0004;
const KEYEVENTF_SCANCODE: u32 = 0x0008;
const IDC_ARROW: *const u16 = 32512usize as *const u16;
const MB_OK: u32 = 0x00000000;
const MB_ICONWARNING: u32 = 0x00000030;
const FW_NORMAL: i32 = 400;
const FW_SEMIBOLD: i32 = 600;
const DEFAULT_CHARSET: u32 = 1;
const CLEARTYPE_QUALITY: u32 = 5;
const VK_RCONTROL: u32 = 0xA3;
const VK_RMENU: u32 = 0xA5;
const VK_LCONTROL: u32 = 0xA2;
const VK_LMENU: u32 = 0xA4;
const VK_CONTROL: u32 = 0x11;
const VK_MENU: u32 = 0x12;
const VK_V: u32 = 0x56;
const VK_INSERT: u32 = 0x2D;
const VK_F1: u32 = 0x70;
const VK_F2: u32 = 0x71;
const CF_UNICODETEXT: u32 = 13;
const MAPVK_VK_TO_VSC: u32 = 0;
const SM_CXSCREEN: i32 = 0;
const SM_CYSCREEN: i32 = 1;
const TIMER_DOCK: usize = 30;
const DOCK_EDGE_RIGHT: i32 = 0;
const DOCK_EDGE_TOP: i32 = 1;
const DOCK_PEEK_W: i32 = 40;
const DOCK_HOT_ZONE: i32 = 16;
const DOCK_TAIL_H: i32 = 142;
const DOCK_HANDLE_W: i32 = 44;
const DOCK_HANDLE_H: i32 = 142;
const DOCK_ANIM_MIN_STEP: i32 = 1;
const DOCK_ANIM_MAX_STEP: i32 = 34;
const TRAY_ICON_ID: u32 = 1;
const NIM_ADD: u32 = 0;
const NIM_DELETE: u32 = 2;
const NIM_SETVERSION: u32 = 4;
const NIF_MESSAGE: u32 = 0x00000001;
const NIF_ICON: u32 = 0x00000002;
const NIF_TIP: u32 = 0x00000004;
const NIF_SHOWTIP: u32 = 0x00000080;
const NOTIFYICON_VERSION_4: u32 = 4;
const MF_STRING: u32 = 0x00000000;
const MF_CHECKED: u32 = 0x00000008;
const MF_SEPARATOR: u32 = 0x00000800;
const TPM_RIGHTBUTTON: u32 = 0x0002;
const TPM_RETURNCMD: u32 = 0x0100;
const GWL_STYLE: i32 = -16;
const GWL_EXSTYLE: i32 = -20;
const MOUSEEVENTF_MOVE: u32 = 0x0001;
const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
const MOUSEEVENTF_LEFTUP: u32 = 0x0004;
const MOUSEEVENTF_ABSOLUTE: u32 = 0x8000;
const HWND_TOP: Hwnd = 0;
const SW_HIDE: i32 = 0;
const SW_SHOW: i32 = 5;
const SW_RESTORE: i32 = 9;
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_FRAMECHANGED: u32 = 0x0020;
const GCLP_HBRBACKGROUND: i32 = -10;
const ODS_SELECTED: u32 = 0x0001;
const ODS_CHECKED: u32 = 0x0008;
const ODS_FOCUS: u32 = 0x0010;
const DT_CENTER: u32 = 0x0001;
const DT_VCENTER: u32 = 0x0004;
const DT_WORDBREAK: u32 = 0x0010;
const DT_SINGLELINE: u32 = 0x0020;
const TRANSPARENT: i32 = 1;
const NULL_PEN: i32 = 8;
const PS_SOLID: i32 = 0;
const IMAGE_ICON: u32 = 1;
const ICON_SMALL: Wparam = 0;
const ICON_BIG: Wparam = 1;
const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;
const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
const DWMWA_BORDER_COLOR: u32 = 34;
const DWMWA_CAPTION_COLOR: u32 = 35;
const DWMWA_TEXT_COLOR: u32 = 36;
const DWMWCP_ROUND: i32 = 2;
const CREATE_NO_WINDOW: u32 = 0x08000000;
const OFN_FILEMUSTEXIST: u32 = 0x00001000;
const OFN_PATHMUSTEXIST: u32 = 0x00000800;
const OFN_NOCHANGEDIR: u32 = 0x00000008;

fn main() {
    if std::env::args().any(|arg| arg == "--init-disease-templates") {
        ensure_editable_text_files();
        return;
    }
    if std::env::args().any(|arg| arg == "--cleanup-old-exes") {
        cleanup_old_executables();
        return;
    }
    unsafe {
        let common_controls = InitCommonControlsEx {
            size: size_of::<InitCommonControlsEx>() as u32,
            classes: ICC_DATE_CLASSES,
        };
        InitCommonControlsEx(&common_controls);
        let h_instance = GetModuleHandleW(null());
        let app_icon = LoadImageW(h_instance, 1usize as *const u16, IMAGE_ICON, 32, 32, 0);
        let class_name = wide(APP_CLASS);
        let wc = WndClassW {
            style: 0,
            lpfn_wnd_proc: wnd_proc,
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance,
            h_icon: app_icon,
            h_cursor: LoadCursorW(0, IDC_ARROW),
            hbr_background: 6,
            lpsz_menu_name: null(),
            lpsz_class_name: class_name.as_ptr(),
        };
        RegisterClassW(&wc);
        close_previous_instance();
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_APPWINDOW,
            class_name.as_ptr(),
            wide(APP_TITLE).as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            DEFAULT_WINDOW_X,
            DEFAULT_WINDOW_Y,
            DEFAULT_WINDOW_W,
            DEFAULT_WINDOW_H,
            0,
            0,
            h_instance,
            null_mut(),
        );

        ShowWindow(hwnd, 5);
        UpdateWindow(hwnd);

        let mut msg: Msg = zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe fn register_selection_popup_class(h_instance: Hinstance) {
    let class_name = wide(SELECTION_POPUP_CLASS);
    let wc = WndClassW {
        style: 0,
        lpfn_wnd_proc: selection_popup_proc,
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance,
        h_icon: 0,
        h_cursor: LoadCursorW(0, IDC_ARROW),
        hbr_background: 0,
        lpsz_menu_name: null(),
        lpsz_class_name: class_name.as_ptr(),
    };
    RegisterClassW(&wc);
}

extern "system" fn selection_popup_proc(
    hwnd: Hwnd,
    msg: u32,
    w_param: Wparam,
    l_param: Lparam,
) -> Lresult {
    unsafe {
        match msg {
            WM_COMMAND => {
                if APP.hwnd != 0 {
                    SendMessageW(APP.hwnd, WM_COMMAND, w_param, l_param);
                }
                0
            }
            WM_DRAWITEM => {
                if APP.hwnd != 0 {
                    SendMessageW(APP.hwnd, WM_DRAWITEM, w_param, l_param);
                }
                1
            }
            WM_CTLCOLORLISTBOX => {
                apply_popup_theme_to_dc(w_param as Hdc);
                theme_popup_brush() as Lresult
            }
            WM_CTLCOLORSTATIC => {
                apply_popup_theme_to_dc(w_param as Hdc);
                theme_popup_brush() as Lresult
            }
            WM_ERASEBKGND => {
                let mut rect = Rect::default();
                GetClientRect(hwnd, &mut rect);
                let brush = CreateSolidBrush(theme_popup_bg_color());
                FillRect(w_param as Hdc, &rect, brush);
                DeleteObject(brush);
                1
            }
            _ => DefWindowProcW(hwnd, msg, w_param, l_param),
        }
    }
}

extern "system" fn wnd_proc(hwnd: Hwnd, msg: u32, w_param: Wparam, l_param: Lparam) -> Lresult {
    unsafe {
        match msg {
            WM_CREATE => {
                APP.hwnd = hwnd;
                let h_instance = GetModuleHandleW(null());
                let icon_big = LoadImageW(h_instance, 1usize as *const u16, IMAGE_ICON, 32, 32, 0);
                let icon_small =
                    LoadImageW(h_instance, 1usize as *const u16, IMAGE_ICON, 16, 16, 0);
                SendMessageW(hwnd, WM_SETICON, ICON_BIG, icon_big);
                SendMessageW(hwnd, WM_SETICON, ICON_SMALL, icon_small);
                MAIN_FONT = create_font(15, FW_NORMAL);
                TITLE_FONT = create_font(20, FW_SEMIBOLD);
                BUTTON_FONT = create_font(15, FW_SEMIBOLD);
                NOTE_FONT = create_font(12, FW_NORMAL);
                DAY_BRUSH = CreateSolidBrush(rgb(242, 242, 247));
                NIGHT_BRUSH = CreateSolidBrush(rgb(28, 28, 30));
                DAY_CONTROL_BRUSH = CreateSolidBrush(rgb(246, 249, 253));
                NIGHT_CONTROL_BRUSH = CreateSolidBrush(rgb(38, 40, 44));
                DAY_POPUP_BRUSH = CreateSolidBrush(rgb(242, 248, 255));
                NIGHT_POPUP_BRUSH = CreateSolidBrush(rgb(32, 43, 56));
                register_selection_popup_class(h_instance);
                register_settings_window_class(h_instance);
                register_dock_handle_class(h_instance);
                ensure_editable_text_files();
                create_controls(hwnd);
                set_taskbar_visible(TASKBAR_VISIBLE.load(Ordering::SeqCst), false);
                add_tray_icon(hwnd, icon_small);
                if DOCK_ENABLED.load(Ordering::SeqCst) {
                    dock_to_edge(true);
                } else {
                    update_dock_button();
                }
                SetTimer(hwnd, TIMER_DOCK, 40, null_mut());
                KEYBOARD_HOOK = SetWindowsHookExW(WH_KEYBOARD_LL, keyboard_proc, 0, 0);
                if KEYBOARD_HOOK == 0 {
                    set_status("请把光标放在书写病程的第一个字之前。右Ctrl+右Alt监听失败。");
                } else {
                    set_status("请把光标放在书写病程的第一个字之前。右Ctrl+右Alt可停止。");
                }
                0
            }
            WM_COMMAND => {
                let id = loword(w_param as u32) as i32;
                let code = hiword(w_param as u32);
                if id == ID_DISEASE_LIST && code == LBN_SELCHANGE {
                    DISEASE_DROPDOWN_OPEN.store(false, Ordering::SeqCst);
                    show_disease_dropdown(false);
                    set_status("已选择疾病；后续可接入疾病占位符。");
                    return 0;
                }
                if l_param as Hwnd == APP.superior_list && code == LBN_SELCHANGE {
                    if let Some(name) = selected_listbox_text(APP.superior_list) {
                        let target = SUPERIOR_TARGET.load(Ordering::SeqCst);
                        set_superior_button_text(target, &name, false);
                        suspend_docking_after_selection();
                        SUPERIOR_PANEL_OPEN.store(false, Ordering::SeqCst);
                        show_superior_panel(false);
                        save_settings(&read_settings_from_ui());
                        set_status(&format!(
                            "已选择{}：{}。",
                            if target == 0 {
                                "主治上级"
                            } else {
                                "主任上级"
                            },
                            name
                        ));
                    }
                    return 0;
                }
                handle_command(id);
                0
            }
            WM_TRAY_ICON => {
                let event = loword(l_param as u32) as u32;
                if event == WM_LBUTTONDBLCLK {
                    restore_main_window();
                } else if event == WM_RBUTTONUP || event == WM_CONTEXTMENU {
                    show_tray_menu();
                }
                0
            }
            WM_CONTEXTMENU => {
                let target = w_param as Hwnd;
                if let Some(index) = api_doc_index_from_hwnd(target) {
                    API_CONTEXT_DOC_INDEX.store(index as i32, Ordering::SeqCst);
                    show_api_doc_context_menu(index);
                    return 0;
                }
                DefWindowProcW(hwnd, msg, w_param, l_param)
            }
            WM_DRAWITEM => {
                let draw = l_param as *const DrawItemStruct;
                if !draw.is_null() && (*draw).ctl_id as i32 == ID_STATUS {
                    draw_status_panel(&*draw);
                } else {
                    draw_owner_button(draw);
                }
                1
            }
            WM_PAINT => {
                if DOCK_REGION_HIDDEN.load(Ordering::SeqCst) {
                    let mut paint = PaintStruct {
                        hdc: 0,
                        erase: 0,
                        rc_paint: Rect::default(),
                        restore: 0,
                        inc_update: 0,
                        reserved: [0; 32],
                    };
                    let hdc = BeginPaint(hwnd, &mut paint);
                    if hdc != 0 {
                        draw_dock_handle(hdc, hwnd);
                    }
                    EndPaint(hwnd, &paint);
                    return 0;
                }
                DefWindowProcW(hwnd, msg, w_param, l_param)
            }
            WM_ERASEBKGND => {
                let mut rect = Rect::default();
                GetClientRect(hwnd, &mut rect);
                FillRect(w_param as Hdc, &rect, theme_brush());
                1
            }
            WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX => {
                apply_control_theme_to_dc(w_param as Hdc);
                theme_control_brush() as Lresult
            }
            WM_CTLCOLORSTATIC => {
                apply_static_theme_to_dc(w_param as Hdc, l_param as Hwnd);
                theme_brush() as Lresult
            }
            WM_CTLCOLORMSGBOX | WM_CTLCOLORBTN => {
                apply_theme_to_dc(w_param as Hdc);
                theme_brush() as Lresult
            }
            WM_SIZE => {
                if w_param == SIZE_MINIMIZED {
                    minimize_to_tray();
                    return 0;
                }
                layout_footer(hwnd);
                0
            }
            WM_DISPLAYCHANGE | WM_DPICHANGED => {
                if DOCK_ENABLED.load(Ordering::SeqCst) {
                    sync_dock_to_current_display();
                }
                0
            }
            WM_TIMER => {
                if w_param == TIMER_DOCK {
                    update_dock_state();
                    return 0;
                }
                DefWindowProcW(hwnd, msg, w_param, l_param)
            }
            WM_DESTROY => {
                save_settings(&read_settings_from_ui());
                STOP_TYPING.store(true, Ordering::SeqCst);
                KillTimer(hwnd, TIMER_DOCK);
                remove_tray_icon(hwnd);
                if KEYBOARD_HOOK != 0 {
                    UnhookWindowsHookEx(KEYBOARD_HOOK);
                    KEYBOARD_HOOK = 0;
                }
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, w_param, l_param),
        }
    }
}

unsafe fn register_settings_window_class(h_instance: Hinstance) {
    let class_name = wide(SETTINGS_WINDOW_CLASS);
    let wc = WndClassW {
        style: 0,
        lpfn_wnd_proc: settings_window_proc,
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance,
        h_icon: 0,
        h_cursor: LoadCursorW(0, IDC_ARROW),
        hbr_background: 0,
        lpsz_menu_name: null(),
        lpsz_class_name: class_name.as_ptr(),
    };
    RegisterClassW(&wc);
}

extern "system" fn settings_window_proc(
    hwnd: Hwnd,
    msg: u32,
    w_param: Wparam,
    l_param: Lparam,
) -> Lresult {
    unsafe {
        match msg {
            WM_CREATE => {
                SETTINGS_DIALOG.hwnd = hwnd;
                create_settings_controls(hwnd);
                0
            }
            WM_COMMAND => {
                let id = loword(w_param as u32) as i32;
                handle_settings_command(id);
                0
            }
            WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX => {
                apply_control_theme_to_dc(w_param as Hdc);
                theme_control_brush() as Lresult
            }
            WM_CTLCOLORSTATIC => {
                apply_static_theme_to_dc(w_param as Hdc, l_param as Hwnd);
                theme_brush() as Lresult
            }
            WM_DRAWITEM => {
                draw_owner_button(l_param as *const DrawItemStruct);
                1
            }
            WM_ERASEBKGND => {
                let mut rect = Rect::default();
                GetClientRect(hwnd, &mut rect);
                FillRect(w_param as Hdc, &rect, theme_brush());
                1
            }
            WM_CLOSE => {
                ShowWindow(hwnd, SW_HIDE);
                0
            }
            WM_DESTROY => {
                SETTINGS_DIALOG.hwnd = 0;
                SETTINGS_DIALOG.medical_path = 0;
                SETTINGS_DIALOG.order_path = 0;
                SETTINGS_DIALOG.clinical_res_1080 = 0;
                SETTINGS_DIALOG.clinical_res_2160 = 0;
                0
            }
            _ => DefWindowProcW(hwnd, msg, w_param, l_param),
        }
    }
}

unsafe fn create_controls(hwnd: Hwnd) {
    APP.logo = CreateWindowExW(
        0,
        wide("STATIC").as_ptr(),
        wide("").as_ptr(),
        WS_CHILD | WS_VISIBLE | SS_ICON,
        4,
        16,
        20,
        20,
        hwnd,
        0,
        0,
        null_mut(),
    );
    let logo_icon = LoadImageW(
        GetModuleHandleW(null()),
        1usize as *const u16,
        IMAGE_ICON,
        20,
        20,
        0,
    );
    SendMessageW(APP.logo, STM_SETICON, logo_icon as Wparam, 0);
    APP.header = label(hwnd, "神外小助手", 30, 14, 120, 24, true);
    apply_font(APP.header, TITLE_FONT);
    APP.dock_toggle = button(hwnd, ID_DOCK_TOGGLE, "吸附", 156, 12, 64, 28);
    APP.theme_toggle = button(hwnd, ID_THEME_TOGGLE, "夜间", 226, 12, 64, 28);
    APP.cleanup_button = button(hwnd, ID_CLEANUP_RUNNING_APPS, "清理", 202, 780, 48, 24);
    APP.settings_button = button(hwnd, ID_SETTINGS_OPEN_MAIN, "⚙", 258, 780, 28, 24);
    APP.tab_templates = tab_button(hwnd, ID_TAB_TEMPLATES, "病程模板", 20, 68, 132, 32, true);
    APP.tab_shortcuts = tab_button(hwnd, ID_TAB_SHORTCUTS, "快捷键", 152, 68, 132, 32, false);
    APP.tab_api = 0;
    APP.divider = label(hwnd, "", 20, 101, 270, 18, false);

    let mut y = 122;
    register_template_control(button(hwnd, ID_FIRST_COURSE, "首次病程", 20, y, 132, 38));
    register_template_control(button(hwnd, ID_SENIOR_FIRST, "首次查房", 158, y, 132, 38));
    y += 48;
    register_template_control(button(hwnd, ID_PREOP_SUMMARY, "术前小结", 20, y, 132, 38));
    register_template_control(button(
        hwnd,
        ID_PREOP_DISCUSSION,
        "术前讨论",
        158,
        y,
        132,
        38,
    ));
    y += 48;

    register_template_control(button(hwnd, ID_POST_FIRST, "术后首程", 20, y, 270, 38));
    y += 44;
    register_template_control(button(
        hwnd,
        ID_POST_SENIOR_FIRST,
        "术后首次上级查房",
        20,
        y,
        270,
        38,
    ));
    y += 44;
    register_template_control(button(hwnd, ID_POST_DAILY, "术后日常", 20, y, 270, 38));
    register_template_control(note_label(hwnd, "日常病程记录模板", 34, y + 39, 230, 18));
    y += 66;

    register_template_control(note_label(hwnd, "出院材料", 34, y, 230, 18));
    y += 20;
    register_template_control(button(hwnd, ID_DISCHARGE_CERT, "诊断证明", 20, y, 132, 38));
    register_template_control(button(
        hwnd,
        ID_DISCHARGE_RECORD,
        "出院记录",
        158,
        y,
        132,
        38,
    ));
    y += 48;

    APP.disease_toggle = button(hwnd, ID_DISEASE_TOGGLE, "疾病：通用 ▼", 20, y + 8, 270, 32);
    register_template_control(APP.disease_toggle);
    APP.disease_popup = selection_popup(hwnd, 276, 436);
    APP.disease_list = popup_listbox(APP.disease_popup, ID_DISEASE_LIST, 270, 430, &DISEASE_ITEMS);
    show_disease_dropdown(false);
    y += 48;

    register_template_control(label(hwnd, "病情选项", 20, y + 8, 260, 24, true));
    APP.htn = checkbox(hwnd, ID_HTN, "高血压", 20, y + 40, 112, 26);
    register_template_control(APP.htn);
    APP.dm = checkbox(hwnd, ID_DM, "糖尿病", 162, y + 40, 112, 26);
    register_template_control(APP.dm);
    APP.fast_hr = checkbox(hwnd, ID_FAST_HR, "心率快", 20, y + 72, 112, 26);
    register_template_control(APP.fast_hr);
    APP.severe = checkbox(hwnd, ID_SEVERE, "病重", 162, y + 72, 112, 26);
    register_template_control(APP.severe);

    APP.skip_reminder = checkbox(
        hwnd,
        ID_SKIP_REMINDER,
        "取消切换光标提醒",
        20,
        y + 108,
        200,
        26,
    );
    register_template_control(APP.skip_reminder);

    register_template_control(button(
        hwnd,
        ID_STOP,
        "停止输入  右Ctrl+右Alt",
        20,
        y + 144,
        270,
        40,
    ));

    APP.save_order_hotkey = checkbox(
        hwnd,
        ID_SHORTCUT_SAVE_ORDER,
        "医嘱快捷保存 · 已关闭\nInsert",
        20,
        120,
        270,
        42,
    );
    register_shortcut_control(APP.save_order_hotkey);
    APP.clipboard_auto = checkbox(
        hwnd,
        ID_CLIPBOARD_AUTO,
        "剪贴板自动输入 · 已关闭\nCtrl+Alt+V",
        20,
        166,
        270,
        42,
    );
    register_shortcut_control(APP.clipboard_auto);
    register_shortcut_control(note_label(hwnd, "一键完成临床路径", 34, 218, 230, 18));
    register_shortcut_control(button(
        hwnd,
        ID_SHORTCUT_VT,
        "单个\nAlt+F1",
        20,
        238,
        108,
        44,
    ));
    register_shortcut_control(button(
        hwnd,
        ID_SHORTCUT_CLINICAL_CONTINUE,
        "持续\nAlt+F2",
        136,
        238,
        108,
        44,
    ));
    APP.clinical_loop_count = edit_box(hwnd, ID_CLINICAL_LOOP_COUNT, 252, 247, 38, 26);
    register_shortcut_control(APP.clinical_loop_count);
    register_shortcut_control(note_label(hwnd, "一键启动", 34, 292, 230, 18));
    register_shortcut_control(button(
        hwnd,
        ID_SHORTCUT_MEDICAL_LAUNCH,
        "病历系统",
        20,
        310,
        132,
        36,
    ));
    register_shortcut_control(button(
        hwnd,
        ID_SHORTCUT_ORDER_LAUNCH,
        "医嘱系统",
        158,
        310,
        132,
        36,
    ));
    register_shortcut_control(button(
        hwnd,
        ID_SHORTCUT_REMOTE_SIGN,
        "服务端一键签名",
        20,
        354,
        270,
        34,
    ));
    register_shortcut_control(note_label(hwnd, "账号", 34, 402, 60, 18));
    APP.login_user = edit_box(hwnd, ID_LOGIN_USER, 104, 399, 166, 22);
    register_shortcut_control(APP.login_user);
    register_shortcut_control(note_label(hwnd, "密码", 34, 436, 60, 18));
    APP.login_pass = edit_box(hwnd, ID_LOGIN_PASS, 104, 433, 166, 22);
    register_shortcut_control(APP.login_pass);
    register_shortcut_control(button(hwnd, ID_ACCOUNT_QUERY, "账号查询", 20, 468, 132, 36));
    register_shortcut_control(button(
        hwnd,
        ID_NURSING_LAUNCH,
        "远卓护理",
        158,
        468,
        132,
        36,
    ));
    register_shortcut_control(note_label(hwnd, "病房楼", 34, 514, 60, 18));
    APP.ward_building = button(hwnd, ID_WARD_BUILDING, "楼栋：C", 88, 508, 86, 30);
    register_shortcut_control(APP.ward_building);
    register_shortcut_control(note_label(hwnd, "层数", 184, 514, 42, 18));
    APP.ward_floor = button(hwnd, ID_WARD_FLOOR, "5楼", 224, 508, 66, 30);
    register_shortcut_control(APP.ward_floor);
    register_shortcut_control(button(
        hwnd,
        ID_SHORTCUT_CREATE_ALL,
        "一键创建术前病程",
        20,
        550,
        270,
        36,
    ));
    register_shortcut_control(note_label(hwnd, "入院日期", 34, 595, 66, 18));
    APP.create_all_base_time = date_picker(hwnd, ID_CREATE_ALL_BASE_TIME, 98, 588, 192, 30);
    register_shortcut_control(APP.create_all_base_time);
    APP.attending_superior = button(
        hwnd,
        ID_ATTENDING_SUPERIOR,
        "主治：裴傲 ▼",
        20,
        628,
        132,
        34,
    );
    register_shortcut_control(APP.attending_superior);
    APP.chief_superior = button(hwnd, ID_CHIEF_SUPERIOR, "主任：裴傲 ▼", 158, 628, 132, 34);
    register_shortcut_control(APP.chief_superior);
    register_shortcut_control(button(
        hwnd,
        ID_SHORTCUT_CREATE_POSTOP,
        "一键创建术后病程",
        20,
        674,
        270,
        36,
    ));
    register_shortcut_control(note_label(hwnd, "手术日期", 28, 727, 70, 18));
    APP.postop_base_time = date_picker(hwnd, ID_POSTOP_BASE_TIME, 98, 720, 192, 30);
    register_shortcut_control(APP.postop_base_time);
    APP.startup_check = 0;

    register_api_control(note_label(hwnd, "患者", 24, 122, 42, 18));
    APP.api_patient = edit_box(hwnd, ID_API_PATIENT, 68, 116, 150, 28);
    register_api_control(APP.api_patient);
    APP.api_patient_new = button(hwnd, ID_API_PATIENT_NEW, "新建", 228, 116, 58, 28);
    register_api_control(APP.api_patient_new);
    register_api_control(button(hwnd, ID_API_PATIENT_ADD, "增加", 294, 116, 58, 28));
    register_api_control(button(
        hwnd,
        ID_API_PATIENT_DELETE,
        "删除",
        360,
        116,
        58,
        28,
    ));
    register_api_control(note_label(hwnd, "文书", 24, 158, 42, 18));
    APP.api_doc_type = edit_box(hwnd, ID_API_DOC_TYPE, 68, 152, 170, 28);
    register_api_control(APP.api_doc_type);
    SetWindowTextW(APP.api_doc_type, wide("首次病程记录").as_ptr());
    register_api_control(note_label(hwnd, "口语化描述", 24, 196, 100, 18));
    APP.api_description = multiline_edit_box(hwnd, ID_API_DESCRIPTION, 24, 218, 286, 176);
    register_api_control(APP.api_description);
    register_api_control(note_label(hwnd, "提示词模板", 24, 410, 100, 18));
    APP.api_template_prompt = multiline_edit_box(hwnd, ID_API_TEMPLATE_PROMPT, 24, 432, 286, 164);
    register_api_control(APP.api_template_prompt);
    APP.api_send = button(hwnd, ID_API_SEND, "生成全部模板", 24, 612, 138, 34);
    register_api_control(APP.api_send);
    APP.api_type_result = button(hwnd, ID_API_TYPE_RESULT, "写回选中模板", 172, 612, 138, 34);
    register_api_control(APP.api_type_result);
    register_api_control(note_label(hwnd, "模板生成状态", 338, 158, 150, 18));
    for index in 0..API_DOC_COUNT {
        let row = button(
            hwnd,
            ID_API_DOC_BASE + index as i32,
            &api_doc_row_text(index, false),
            338,
            184 + index as i32 * 44,
            360,
            36,
        );
        APP.api_doc_rows[index] = row;
        register_api_control(row);
    }

    let superior_names: Vec<String> = ACCOUNT_QUERY_ROWS
        .iter()
        .map(|(name, _)| (*name).to_string())
        .collect();
    APP.superior_header = 0;
    APP.superior_popup = selection_popup(hwnd, 276, 332);
    APP.superior_list = popup_listbox(
        APP.superior_popup,
        ID_SUPERIOR_LIST,
        270,
        326,
        &superior_names,
    );
    show_superior_panel(false);
    APP.account_popup = create_account_query_popup(hwnd);
    ShowWindow(APP.account_popup, 0);

    APP.status = CreateWindowExW(
        0,
        wide("STATIC").as_ptr(),
        wide("就绪").as_ptr(),
        WS_CHILD | WS_VISIBLE | SS_OWNERDRAW,
        20,
        588,
        270,
        34,
        hwnd,
        ID_STATUS as Hmenu,
        0,
        null_mut(),
    );
    apply_font(APP.status, NOTE_FONT);
    APP.version = CreateWindowExW(
        0,
        wide("STATIC").as_ptr(),
        wide(&format!("{}  更新：{}", APP_VERSION, APP_UPDATED_AT)).as_ptr(),
        WS_CHILD | WS_VISIBLE,
        20,
        626,
        270,
        20,
        hwnd,
        ID_VERSION as Hmenu,
        0,
        null_mut(),
    );
    apply_font(APP.version, NOTE_FONT);
    load_settings_into_ui();
    show_template_page(true);
    layout_footer(hwnd);
}

unsafe fn button(hwnd: Hwnd, id: i32, text: &str, x: i32, y: i32, w: i32, h: i32) -> Hwnd {
    let control = CreateWindowExW(
        0,
        wide("BUTTON").as_ptr(),
        wide(text).as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_OWNERDRAW,
        x,
        y,
        w,
        h,
        hwnd,
        id as Hmenu,
        0,
        null_mut(),
    );
    apply_font(control, BUTTON_FONT);
    control
}

unsafe fn tab_button(
    hwnd: Hwnd,
    id: i32,
    text: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    first: bool,
) -> Hwnd {
    let group_style = if first { WS_GROUP } else { 0 };
    let control = CreateWindowExW(
        0,
        wide("BUTTON").as_ptr(),
        wide(text).as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | group_style | BS_OWNERDRAW,
        x,
        y,
        w,
        h,
        hwnd,
        id as Hmenu,
        0,
        null_mut(),
    );
    apply_font(control, BUTTON_FONT);
    control
}

unsafe fn checkbox(hwnd: Hwnd, id: i32, text: &str, x: i32, y: i32, w: i32, h: i32) -> Hwnd {
    let control = CreateWindowExW(
        0,
        wide("BUTTON").as_ptr(),
        wide(text).as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_OWNERDRAW,
        x,
        y,
        w,
        h,
        hwnd,
        id as Hmenu,
        0,
        null_mut(),
    );
    apply_font(control, MAIN_FONT);
    control
}

unsafe fn edit_box(hwnd: Hwnd, id: i32, x: i32, y: i32, w: i32, h: i32) -> Hwnd {
    let control = CreateWindowExW(
        WS_EX_STATICEDGE,
        wide("EDIT").as_ptr(),
        wide("").as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | ES_LEFT,
        x,
        y,
        w,
        h,
        hwnd,
        id as Hmenu,
        0,
        null_mut(),
    );
    apply_font(control, MAIN_FONT);
    let margins = 7u32 | (7u32 << 16);
    SendMessageW(
        control,
        EM_SETMARGINS,
        (EC_LEFTMARGIN | EC_RIGHTMARGIN) as Wparam,
        margins as Lparam,
    );
    apply_rounded_region(control, w, h, 10);
    control
}

unsafe fn multiline_edit_box(hwnd: Hwnd, id: i32, x: i32, y: i32, w: i32, h: i32) -> Hwnd {
    let control = CreateWindowExW(
        WS_EX_STATICEDGE,
        wide("EDIT").as_ptr(),
        wide("").as_ptr(),
        WS_CHILD
            | WS_VISIBLE
            | WS_TABSTOP
            | WS_VSCROLL
            | ES_LEFT
            | ES_MULTILINE
            | ES_AUTOVSCROLL
            | ES_WANTRETURN,
        x,
        y,
        w,
        h,
        hwnd,
        id as Hmenu,
        0,
        null_mut(),
    );
    apply_font(control, MAIN_FONT);
    let margins = 7u32 | (7u32 << 16);
    SendMessageW(
        control,
        EM_SETMARGINS,
        (EC_LEFTMARGIN | EC_RIGHTMARGIN) as Wparam,
        margins as Lparam,
    );
    apply_rounded_region(control, w, h, 10);
    control
}

unsafe fn create_account_query_popup(owner: Hwnd) -> Hwnd {
    let width = 350;
    let height = 456;
    let popup = selection_popup(owner, width, height);
    label(popup, "账号查询", 20, 14, 200, 28, true);
    note_label(popup, "神经外科", 244, 20, 84, 20);
    let name_header = label(popup, "姓名", 38, 52, 110, 24, false);
    let code_header = label(popup, "工号", 218, 52, 92, 24, false);
    apply_font(name_header, BUTTON_FONT);
    apply_font(code_header, BUTTON_FONT);

    for (index, (name, code)) in ACCOUNT_QUERY_ROWS.iter().enumerate() {
        let y = 80 + index as i32 * 23;
        label(popup, name, 38, y, 120, 22, false);
        label(popup, code, 218, y, 92, 22, false);
    }

    button(popup, ID_ACCOUNT_DIALOG_CLOSE, "关闭", 116, 412, 118, 32);
    popup
}

unsafe fn create_settings_controls(hwnd: Hwnd) {
    let settings = load_settings();
    let title = label(hwnd, "设置", 20, 18, 280, 30, true);
    apply_font(title, TITLE_FONT);

    note_label(hwnd, "病历系统", 22, 58, 72, 22);
    SETTINGS_DIALOG.medical_path = edit_box(hwnd, ID_SETTINGS_MEDICAL_PATH, 92, 54, 292, 30);
    SetWindowTextW(
        SETTINGS_DIALOG.medical_path,
        wide(&setting_or_default(
            &settings.medical_system_path,
            default_medical_system_path(),
        ))
        .as_ptr(),
    );
    button(hwnd, ID_SETTINGS_BROWSE_MEDICAL, "浏览...", 394, 53, 76, 32);

    note_label(hwnd, "医嘱系统", 22, 100, 72, 22);
    SETTINGS_DIALOG.order_path = edit_box(hwnd, ID_SETTINGS_ORDER_PATH, 92, 96, 292, 30);
    SetWindowTextW(
        SETTINGS_DIALOG.order_path,
        wide(&setting_or_default(
            &settings.order_system_path,
            default_order_system_path(),
        ))
        .as_ptr(),
    );
    button(hwnd, ID_SETTINGS_BROWSE_ORDER, "浏览...", 394, 95, 76, 32);

    note_label(hwnd, "护理系统启动文件", 22, 144, 100, 22);
    SETTINGS_DIALOG.nursing_path = edit_box(hwnd, ID_SETTINGS_NURSING_PATH, 92, 140, 292, 30);
    SetWindowTextW(
        SETTINGS_DIALOG.nursing_path,
        wide(&setting_or_default(
            &settings.nursing_system_path,
            default_nursing_system_path(),
        ))
        .as_ptr(),
    );
    button(
        hwnd,
        ID_SETTINGS_BROWSE_NURSING,
        "浏览...",
        394,
        139,
        76,
        32,
    );

    note_label(hwnd, "签名服务地址", 22, 184, 100, 22);
    SETTINGS_DIALOG.remote_sign_server =
        edit_box(hwnd, ID_SETTINGS_REMOTE_SIGN_SERVER, 122, 180, 190, 30);
    SetWindowTextW(
        SETTINGS_DIALOG.remote_sign_server,
        wide(&configured_remote_sign_server(&settings)).as_ptr(),
    );
    note_label(hwnd, "端口", 320, 184, 42, 22);
    SETTINGS_DIALOG.remote_sign_port =
        edit_box(hwnd, ID_SETTINGS_REMOTE_SIGN_PORT, 360, 180, 110, 30);
    SetWindowTextW(
        SETTINGS_DIALOG.remote_sign_port,
        wide(&configured_remote_sign_port(&settings).to_string()).as_ptr(),
    );

    note_label(hwnd, "自动隐藏延迟", 22, 226, 96, 22);
    SETTINGS_DIALOG.dock_hide_delay = edit_box(hwnd, ID_SETTINGS_DOCK_HIDE_DELAY, 122, 222, 88, 30);
    SetWindowTextW(
        SETTINGS_DIALOG.dock_hide_delay,
        wide(&settings.dock_hide_delay_ms.to_string()).as_ptr(),
    );
    note_label(hwnd, "毫秒，0为立即回缩", 218, 227, 150, 22);

    note_label(hwnd, "临床路径分辨率", 22, 274, 112, 22);
    SETTINGS_DIALOG.clinical_res_1080 =
        button(hwnd, ID_SETTINGS_RES_1080, "1080p", 140, 270, 96, 30);
    SETTINGS_DIALOG.clinical_res_2160 =
        button(hwnd, ID_SETTINGS_RES_2160, "2160p", 246, 270, 96, 30);
    set_checkbox(
        ID_SETTINGS_RES_1080,
        SETTINGS_DIALOG.clinical_res_1080,
        settings.clinical_path_resolution_profile == 0,
    );
    set_checkbox(
        ID_SETTINGS_RES_2160,
        SETTINGS_DIALOG.clinical_res_2160,
        settings.clinical_path_resolution_profile == 1,
    );

    note_label(hwnd, "EMR注册界面", 22, 318, 96, 22);
    SETTINGS_DIALOG.launch_alt_f4 = checkbox(hwnd, ID_SETTINGS_ALT_F4, "有", 120, 314, 64, 30);
    SETTINGS_DIALOG.launch_alt_f4_no =
        checkbox(hwnd, ID_SETTINGS_ALT_F4_NO, "无", 190, 314, 64, 30);
    set_checkbox(
        ID_SETTINGS_ALT_F4,
        SETTINGS_DIALOG.launch_alt_f4,
        settings.launch_alt_f4,
    );
    set_checkbox(
        ID_SETTINGS_ALT_F4_NO,
        SETTINGS_DIALOG.launch_alt_f4_no,
        !settings.launch_alt_f4,
    );
    SETTINGS_DIALOG.startup_check =
        checkbox(hwnd, ID_STARTUP_CHECK, "开机自启动", 22, 364, 120, 26);
    set_checkbox(
        ID_STARTUP_CHECK,
        SETTINGS_DIALOG.startup_check,
        is_startup_enabled(),
    );
    SETTINGS_DIALOG.auto_remote_sign = checkbox(
        hwnd,
        ID_SETTINGS_AUTO_REMOTE_SIGN,
        "启动病历/医嘱后自动一键签名",
        158,
        364,
        310,
        26,
    );
    set_checkbox(
        ID_SETTINGS_AUTO_REMOTE_SIGN,
        SETTINGS_DIALOG.auto_remote_sign,
        settings.auto_remote_sign,
    );

    button(hwnd, ID_SETTINGS_SAVE, "保存", 264, 422, 92, 34);
    button(hwnd, ID_SETTINGS_CANCEL, "取消", 372, 422, 92, 34);
    set_theme(DARK_THEME.load(Ordering::SeqCst));
}

unsafe fn show_settings_window() {
    let w = 492;
    let h = 490;
    if SETTINGS_DIALOG.hwnd != 0 && IsWindow(SETTINGS_DIALOG.hwnd) != 0 {
        refresh_settings_window_values();
        ShowWindow(SETTINGS_DIALOG.hwnd, SW_SHOW);
        BringWindowToTop(SETTINGS_DIALOG.hwnd);
        SetForegroundWindow(SETTINGS_DIALOG.hwnd);
        return;
    }

    let mut parent_rect = Rect::default();
    let (x, y) = if APP.hwnd != 0 && GetWindowRect(APP.hwnd, &mut parent_rect) != 0 {
        (parent_rect.left - w - 14, (parent_rect.top + 80).max(20))
    } else {
        (120, 120)
    };
    let hwnd = CreateWindowExW(
        WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
        wide(SETTINGS_WINDOW_CLASS).as_ptr(),
        wide("神外小助手设置").as_ptr(),
        WS_OVERLAPPEDWINDOW | WS_VISIBLE,
        x.max(20),
        y,
        w,
        h,
        APP.hwnd,
        0,
        0,
        null_mut(),
    );
    if hwnd != 0 {
        let corner = DWMWCP_ROUND;
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &corner as *const _ as *const _,
            size_of::<i32>() as u32,
        );
        apply_settings_window_chrome(hwnd, DARK_THEME.load(Ordering::SeqCst));
        SetForegroundWindow(hwnd);
    }
}

unsafe fn refresh_settings_window_values() {
    let settings = load_settings();
    if SETTINGS_DIALOG.medical_path != 0 {
        SetWindowTextW(
            SETTINGS_DIALOG.medical_path,
            wide(&setting_or_default(
                &settings.medical_system_path,
                default_medical_system_path(),
            ))
            .as_ptr(),
        );
    }
    if SETTINGS_DIALOG.order_path != 0 {
        SetWindowTextW(
            SETTINGS_DIALOG.order_path,
            wide(&setting_or_default(
                &settings.order_system_path,
                default_order_system_path(),
            ))
            .as_ptr(),
        );
    }
    if SETTINGS_DIALOG.nursing_path != 0 {
        SetWindowTextW(
            SETTINGS_DIALOG.nursing_path,
            wide(&setting_or_default(
                &settings.nursing_system_path,
                default_nursing_system_path(),
            ))
            .as_ptr(),
        );
    }
    if SETTINGS_DIALOG.remote_sign_server != 0 {
        SetWindowTextW(
            SETTINGS_DIALOG.remote_sign_server,
            wide(&configured_remote_sign_server(&settings)).as_ptr(),
        );
    }
    if SETTINGS_DIALOG.remote_sign_port != 0 {
        SetWindowTextW(
            SETTINGS_DIALOG.remote_sign_port,
            wide(&configured_remote_sign_port(&settings).to_string()).as_ptr(),
        );
    }
    if SETTINGS_DIALOG.dock_hide_delay != 0 {
        SetWindowTextW(
            SETTINGS_DIALOG.dock_hide_delay,
            wide(&settings.dock_hide_delay_ms.to_string()).as_ptr(),
        );
    }
    set_checkbox(
        ID_SETTINGS_RES_1080,
        SETTINGS_DIALOG.clinical_res_1080,
        settings.clinical_path_resolution_profile == 0,
    );
    set_checkbox(
        ID_SETTINGS_RES_2160,
        SETTINGS_DIALOG.clinical_res_2160,
        settings.clinical_path_resolution_profile == 1,
    );
    set_checkbox(
        ID_SETTINGS_ALT_F4,
        SETTINGS_DIALOG.launch_alt_f4,
        settings.launch_alt_f4,
    );
    set_checkbox(
        ID_SETTINGS_ALT_F4_NO,
        SETTINGS_DIALOG.launch_alt_f4_no,
        !settings.launch_alt_f4,
    );
    set_checkbox(
        ID_STARTUP_CHECK,
        SETTINGS_DIALOG.startup_check,
        is_startup_enabled(),
    );
    set_checkbox(
        ID_SETTINGS_AUTO_REMOTE_SIGN,
        SETTINGS_DIALOG.auto_remote_sign,
        settings.auto_remote_sign,
    );
}

unsafe fn handle_settings_command(id: i32) {
    match id {
        ID_SETTINGS_RES_1080 => {
            set_checkbox(
                ID_SETTINGS_RES_1080,
                SETTINGS_DIALOG.clinical_res_1080,
                true,
            );
            set_checkbox(
                ID_SETTINGS_RES_2160,
                SETTINGS_DIALOG.clinical_res_2160,
                false,
            );
        }
        ID_SETTINGS_RES_2160 => {
            set_checkbox(
                ID_SETTINGS_RES_1080,
                SETTINGS_DIALOG.clinical_res_1080,
                false,
            );
            set_checkbox(
                ID_SETTINGS_RES_2160,
                SETTINGS_DIALOG.clinical_res_2160,
                true,
            );
        }
        ID_SETTINGS_ALT_F4 => {
            set_checkbox(ID_SETTINGS_ALT_F4, SETTINGS_DIALOG.launch_alt_f4, true);
            set_checkbox(
                ID_SETTINGS_ALT_F4_NO,
                SETTINGS_DIALOG.launch_alt_f4_no,
                false,
            );
        }
        ID_SETTINGS_ALT_F4_NO => {
            set_checkbox(ID_SETTINGS_ALT_F4, SETTINGS_DIALOG.launch_alt_f4, false);
            set_checkbox(
                ID_SETTINGS_ALT_F4_NO,
                SETTINGS_DIALOG.launch_alt_f4_no,
                true,
            );
        }
        ID_STARTUP_CHECK => {
            let enabled = !is_startup_enabled();
            if set_startup_enabled(enabled) {
                set_checkbox(ID_STARTUP_CHECK, SETTINGS_DIALOG.startup_check, enabled);
            }
        }
        ID_SETTINGS_AUTO_REMOTE_SIGN => {
            let enabled = !checkbox_checked(ID_SETTINGS_AUTO_REMOTE_SIGN);
            set_checkbox(
                ID_SETTINGS_AUTO_REMOTE_SIGN,
                SETTINGS_DIALOG.auto_remote_sign,
                enabled,
            );
        }
        ID_SETTINGS_BROWSE_MEDICAL => {
            if let Some(path) = choose_executable_file(
                SETTINGS_DIALOG.hwnd,
                "选择病历系统启动文件",
                &get_window_text(SETTINGS_DIALOG.medical_path),
            ) {
                SetWindowTextW(SETTINGS_DIALOG.medical_path, wide(&path).as_ptr());
            }
        }
        ID_SETTINGS_BROWSE_ORDER => {
            if let Some(path) = choose_executable_file(
                SETTINGS_DIALOG.hwnd,
                "选择医嘱系统启动文件",
                &get_window_text(SETTINGS_DIALOG.order_path),
            ) {
                SetWindowTextW(SETTINGS_DIALOG.order_path, wide(&path).as_ptr());
            }
        }
        ID_SETTINGS_BROWSE_NURSING => {
            if let Some(path) = choose_executable_file(
                SETTINGS_DIALOG.hwnd,
                "选择护理系统启动文件",
                &get_window_text(SETTINGS_DIALOG.nursing_path),
            ) {
                SetWindowTextW(SETTINGS_DIALOG.nursing_path, wide(&path).as_ptr());
            }
        }
        ID_SETTINGS_SAVE => {
            let mut settings = if APP.hwnd != 0 {
                read_settings_from_ui()
            } else {
                load_settings()
            };
            settings.medical_system_path = get_window_text(SETTINGS_DIALOG.medical_path);
            settings.order_system_path = get_window_text(SETTINGS_DIALOG.order_path);
            settings.nursing_system_path = get_window_text(SETTINGS_DIALOG.nursing_path);
            settings.remote_sign_server = get_window_text(SETTINGS_DIALOG.remote_sign_server);
            settings.remote_sign_port =
                normalized_remote_sign_port(&get_window_text(SETTINGS_DIALOG.remote_sign_port));
            settings.auto_remote_sign = checkbox_checked(ID_SETTINGS_AUTO_REMOTE_SIGN);
            settings.dock_hide_delay_ms =
                normalized_dock_hide_delay(&get_window_text(SETTINGS_DIALOG.dock_hide_delay));
            settings.clinical_path_resolution_profile =
                usize::from(checkbox_checked(ID_SETTINGS_RES_2160));
            settings.launch_alt_f4 = checkbox_checked(ID_SETTINGS_ALT_F4);
            settings.startup_enabled = checkbox_checked(ID_STARTUP_CHECK);
            DOCK_HIDE_DELAY_MS.store(settings.dock_hide_delay_ms, Ordering::SeqCst);
            CLINICAL_PATH_RESOLUTION_PROFILE
                .store(settings.clinical_path_resolution_profile, Ordering::SeqCst);
            set_startup_enabled(settings.startup_enabled);
            save_settings(&settings);
            ShowWindow(SETTINGS_DIALOG.hwnd, SW_HIDE);
            set_status("设置已保存。");
        }
        ID_SETTINGS_CANCEL => {
            ShowWindow(SETTINGS_DIALOG.hwnd, SW_HIDE);
        }
        _ => {}
    }
}

unsafe fn hwnd_checked(hwnd: Hwnd) -> bool {
    if hwnd == SETTINGS_DIALOG.api_mode {
        return SETTINGS_API_MODE_CHECKED.load(Ordering::SeqCst);
    }
    hwnd != 0 && SendMessageW(hwnd, BM_GETCHECK, 0, 0) == BST_CHECKED
}

unsafe fn set_hwnd_checked(hwnd: Hwnd, checked: bool) {
    if hwnd == 0 {
        return;
    }
    if hwnd == SETTINGS_DIALOG.api_mode {
        SETTINGS_API_MODE_CHECKED.store(checked, Ordering::SeqCst);
        SetWindowTextW(
            hwnd,
            wide(if checked {
                "API模式：打开"
            } else {
                "API模式：关闭"
            })
            .as_ptr(),
        );
        InvalidateRect(hwnd, null(), 1);
        return;
    }
    let check = if checked { BST_CHECKED as Wparam } else { 0 };
    SendMessageW(hwnd, BM_SETCHECK, check, 0);
    InvalidateRect(hwnd, null(), 1);
}

fn normalized_dock_hide_delay(value: &str) -> u64 {
    value.trim().parse::<u64>().unwrap_or(0).min(5_000)
}

unsafe fn choose_executable_file(owner: Hwnd, title: &str, current: &str) -> Option<String> {
    let mut file_buf = [0u16; 1024];
    let current_wide: Vec<u16> = OsStr::new(current).encode_wide().collect();
    let count = current_wide.len().min(file_buf.len() - 1);
    file_buf[..count].copy_from_slice(&current_wide[..count]);

    let filter = wide_double_null(&[
        "程序或脚本 (*.exe;*.bat;*.cmd)",
        "*.exe;*.bat;*.cmd",
        "所有文件 (*.*)",
        "*.*",
    ]);
    let title_w = wide(title);
    let mut ofn: OpenFileNameW = zeroed();
    ofn.l_struct_size = size_of::<OpenFileNameW>() as u32;
    ofn.hwnd_owner = owner;
    ofn.lpstr_filter = filter.as_ptr();
    ofn.lpstr_file = file_buf.as_mut_ptr();
    ofn.n_max_file = file_buf.len() as u32;
    ofn.lpstr_title = title_w.as_ptr();
    ofn.flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR;

    if GetOpenFileNameW(&mut ofn) == 0 {
        return None;
    }
    let len = file_buf.iter().position(|&ch| ch == 0).unwrap_or(0);
    Some(
        OsString::from_wide(&file_buf[..len])
            .to_string_lossy()
            .to_string(),
    )
}

unsafe fn date_picker(hwnd: Hwnd, id: i32, x: i32, y: i32, w: i32, h: i32) -> Hwnd {
    let control = CreateWindowExW(
        WS_EX_STATICEDGE,
        wide("SysDateTimePick32").as_ptr(),
        wide("").as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP,
        x,
        y,
        w,
        h,
        hwnd,
        id as Hmenu,
        0,
        null_mut(),
    );
    apply_font(control, MAIN_FONT);
    let format = wide("yyyy-MM-dd");
    SendMessageW(control, DTM_SETFORMATW, 0, format.as_ptr() as Lparam);
    apply_rounded_region(control, w, h, 10);
    control
}

unsafe fn selection_popup(owner: Hwnd, w: i32, h: i32) -> Hwnd {
    let popup = CreateWindowExW(
        WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
        wide(SELECTION_POPUP_CLASS).as_ptr(),
        wide("").as_ptr(),
        WS_POPUP,
        0,
        0,
        w,
        h,
        owner,
        0,
        0,
        null_mut(),
    );
    let corner = DWMWCP_ROUND;
    DwmSetWindowAttribute(
        popup,
        DWMWA_WINDOW_CORNER_PREFERENCE,
        &corner as *const _ as *const _,
        size_of::<i32>() as u32,
    );
    apply_rounded_region(popup, w, h, 14);
    popup
}

unsafe fn popup_listbox<S: AsRef<str>>(popup: Hwnd, id: i32, w: i32, h: i32, items: &[S]) -> Hwnd {
    let control = CreateWindowExW(
        0,
        wide("LISTBOX").as_ptr(),
        wide("").as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_VSCROLL | LBS_NOTIFY | LBS_NOINTEGRALHEIGHT,
        3,
        3,
        w,
        h,
        popup,
        id as Hmenu,
        0,
        null_mut(),
    );
    apply_font(control, MAIN_FONT);
    apply_rounded_region(control, w, h, 10);
    for item in items {
        let text = wide(item.as_ref());
        SendMessageW(control, LB_ADDSTRING, 0, text.as_ptr() as Lparam);
    }
    SendMessageW(control, LB_SETCURSEL, 0, 0);
    control
}

unsafe fn apply_rounded_region(hwnd: Hwnd, width: i32, height: i32, radius: i32) {
    if hwnd == 0 || width <= 0 || height <= 0 {
        return;
    }
    let region = CreateRoundRectRgn(0, 0, width + 1, height + 1, radius, radius);
    if region != 0 {
        SetWindowRgn(hwnd, region, 1);
    }
}

unsafe fn label(hwnd: Hwnd, text: &str, x: i32, y: i32, w: i32, h: i32, title: bool) -> Hwnd {
    let control = CreateWindowExW(
        0,
        wide("STATIC").as_ptr(),
        wide(text).as_ptr(),
        WS_CHILD | WS_VISIBLE,
        x,
        y,
        w,
        h,
        hwnd,
        0,
        0,
        null_mut(),
    );
    apply_font(control, if title { TITLE_FONT } else { MAIN_FONT });
    control
}

unsafe fn note_label(hwnd: Hwnd, text: &str, x: i32, y: i32, w: i32, h: i32) -> Hwnd {
    let control = CreateWindowExW(
        0,
        wide("STATIC").as_ptr(),
        wide(text).as_ptr(),
        WS_CHILD | WS_VISIBLE,
        x,
        y,
        w,
        h,
        hwnd,
        0,
        0,
        null_mut(),
    );
    apply_font(control, NOTE_FONT);
    control
}

unsafe fn register_template_control(hwnd: Hwnd) {
    if TEMPLATE_CONTROL_COUNT < TEMPLATE_CONTROL_LIMIT {
        TEMPLATE_CONTROLS[TEMPLATE_CONTROL_COUNT] = hwnd;
        TEMPLATE_CONTROL_COUNT += 1;
    }
}

unsafe fn register_shortcut_control(hwnd: Hwnd) {
    if SHORTCUT_CONTROL_COUNT < SHORTCUT_CONTROL_LIMIT {
        SHORTCUT_CONTROLS[SHORTCUT_CONTROL_COUNT] = hwnd;
        SHORTCUT_CONTROL_COUNT += 1;
    }
}

unsafe fn register_api_control(hwnd: Hwnd) {
    if API_CONTROL_COUNT < API_CONTROL_LIMIT {
        API_CONTROLS[API_CONTROL_COUNT] = hwnd;
        API_CONTROL_COUNT += 1;
    }
}

fn api_doc_row_text(index: usize, generated: bool) -> String {
    let name = API_DOC_NAMES.get(index).copied().unwrap_or("模板");
    format!(
        "{}    {}",
        name,
        if generated { "已生成" } else { "未生成" }
    )
}

fn api_doc_is_generated(index: usize) -> bool {
    API_DOC_GENERATED_MASK.load(Ordering::SeqCst) & (1 << index) != 0
}

unsafe fn refresh_api_doc_rows() {
    for index in 0..API_DOC_COUNT {
        let hwnd = APP.api_doc_rows[index];
        if hwnd != 0 {
            SetWindowTextW(
                hwnd,
                wide(&api_doc_row_text(index, api_doc_is_generated(index))).as_ptr(),
            );
            InvalidateRect(hwnd, null(), 1);
        }
    }
}

unsafe fn reset_api_patient_workspace() {
    API_DOC_GENERATED_MASK.store(0, Ordering::SeqCst);
    API_CONTEXT_DOC_INDEX.store(-1, Ordering::SeqCst);
    refresh_api_doc_rows();
}

fn generated_api_doc_text(index: usize) -> String {
    let name = API_DOC_NAMES.get(index).copied().unwrap_or("模板");
    format!("【{}】\r\n此处将写入 API 生成后的针对性病历内容。", name)
}

unsafe fn writeback_api_doc(index: usize) {
    if index >= API_DOC_COUNT {
        return;
    }
    if !api_doc_is_generated(index) {
        set_status("该模板尚未生成，不能写回。");
        return;
    }
    let text = generated_api_doc_text(index);
    STOP_TYPING.store(false, Ordering::SeqCst);
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));
        type_text(&text);
    });
    set_status("2 秒后写回选中的 API 模板。");
}

unsafe fn show_template_page(show_templates: bool) {
    resize_main_window_for_page(DEFAULT_WINDOW_W, DEFAULT_WINDOW_H);
    API_PAGE_VISIBLE.store(false, Ordering::SeqCst);
    TEMPLATE_PAGE_VISIBLE.store(show_templates, Ordering::SeqCst);
    ShowWindow(APP.tab_templates, 5);
    ShowWindow(APP.tab_shortcuts, 5);
    ShowWindow(APP.tab_api, 0);
    ShowWindow(APP.divider, 5);
    SendMessageW(
        APP.tab_templates,
        BM_SETCHECK,
        if show_templates { 1 } else { 0 },
        0,
    );
    SendMessageW(
        APP.tab_shortcuts,
        BM_SETCHECK,
        if show_templates { 0 } else { 1 },
        0,
    );
    InvalidateRect(APP.tab_templates, null(), 1);
    InvalidateRect(APP.tab_shortcuts, null(), 1);
    InvalidateRect(APP.tab_api, null(), 1);
    for i in 0..TEMPLATE_CONTROL_COUNT {
        ShowWindow(TEMPLATE_CONTROLS[i], if show_templates { 5 } else { 0 });
    }
    for i in 0..SHORTCUT_CONTROL_COUNT {
        ShowWindow(SHORTCUT_CONTROLS[i], if show_templates { 0 } else { 5 });
    }
    for i in 0..API_CONTROL_COUNT {
        ShowWindow(API_CONTROLS[i], 0);
    }
    if !show_templates {
        DISEASE_DROPDOWN_OPEN.store(false, Ordering::SeqCst);
        show_disease_dropdown(false);
    } else {
        SUPERIOR_PANEL_OPEN.store(false, Ordering::SeqCst);
        show_superior_panel(false);
    }
}

unsafe fn show_api_page() {
    resize_main_window_for_page(API_WINDOW_W, API_WINDOW_H);
    API_PAGE_VISIBLE.store(true, Ordering::SeqCst);
    TEMPLATE_PAGE_VISIBLE.store(false, Ordering::SeqCst);
    ShowWindow(APP.tab_templates, 0);
    ShowWindow(APP.tab_shortcuts, 0);
    ShowWindow(APP.tab_api, 0);
    ShowWindow(APP.divider, 0);
    DISEASE_DROPDOWN_OPEN.store(false, Ordering::SeqCst);
    SUPERIOR_PANEL_OPEN.store(false, Ordering::SeqCst);
    show_disease_dropdown(false);
    show_superior_panel(false);
    SendMessageW(APP.tab_templates, BM_SETCHECK, 0, 0);
    SendMessageW(APP.tab_shortcuts, BM_SETCHECK, 0, 0);
    SendMessageW(APP.tab_api, BM_SETCHECK, 1, 0);
    InvalidateRect(APP.tab_templates, null(), 1);
    InvalidateRect(APP.tab_shortcuts, null(), 1);
    InvalidateRect(APP.tab_api, null(), 1);
    for i in 0..TEMPLATE_CONTROL_COUNT {
        ShowWindow(TEMPLATE_CONTROLS[i], 0);
    }
    for i in 0..SHORTCUT_CONTROL_COUNT {
        ShowWindow(SHORTCUT_CONTROLS[i], 0);
    }
    for i in 0..API_CONTROL_COUNT {
        ShowWindow(API_CONTROLS[i], 5);
    }
    refresh_api_doc_rows();
}

unsafe fn resize_main_window_for_page(width: i32, height: i32) {
    if APP.hwnd == 0 {
        return;
    }
    let mut rect = Rect::default();
    if GetWindowRect(APP.hwnd, &mut rect) == 0 {
        return;
    }
    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let right = rect.right.min(screen_w);
    let x = if DOCK_EDGE.load(Ordering::SeqCst) == DOCK_EDGE_RIGHT {
        (right - width).max(0)
    } else {
        rect.left.max(0)
    };
    SetWindowPos(APP.hwnd, HWND_TOP, x, rect.top.max(0), width, height, 0);
    update_main_dock_region(true, height);
}

unsafe fn show_disease_dropdown(show: bool) {
    let show = show && TEMPLATE_PAGE_VISIBLE.load(Ordering::SeqCst);
    if show {
        SUPERIOR_PANEL_OPEN.store(false, Ordering::SeqCst);
        show_superior_panel(false);
    }
    if APP.disease_popup != 0 {
        if show {
            position_selection_popup(APP.disease_popup, APP.disease_toggle, 276, 436);
            ShowWindow(APP.disease_popup, 5);
            SetWindowPos(
                APP.disease_popup,
                HWND_TOP,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE,
            );
            SetFocus(APP.disease_list);
        } else {
            ShowWindow(APP.disease_popup, 0);
        }
    }
    if APP.disease_toggle != 0 {
        update_disease_button(show);
    }
}

unsafe fn show_superior_panel(show: bool) {
    let show = show && !TEMPLATE_PAGE_VISIBLE.load(Ordering::SeqCst);
    if show {
        DISEASE_DROPDOWN_OPEN.store(false, Ordering::SeqCst);
        show_disease_dropdown(false);
    }
    if APP.superior_popup != 0 {
        if show {
            let anchor = if SUPERIOR_TARGET.load(Ordering::SeqCst) == 0 {
                APP.attending_superior
            } else {
                APP.chief_superior
            };
            position_selection_popup(APP.superior_popup, anchor, 276, 332);
            ShowWindow(APP.superior_popup, 5);
            SetWindowPos(
                APP.superior_popup,
                HWND_TOP,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE,
            );
            SetFocus(APP.superior_list);
        } else {
            ShowWindow(APP.superior_popup, 0);
        }
    }
    if !show && APP.attending_superior != 0 && APP.chief_superior != 0 {
        set_superior_button_text(0, &selected_superior_name(0), false);
        set_superior_button_text(1, &selected_superior_name(1), false);
    }
}

unsafe fn position_selection_popup(popup: Hwnd, anchor: Hwnd, popup_w: i32, popup_h: i32) {
    if anchor == 0 || popup == 0 {
        return;
    }

    let mut anchor_rect = Rect::default();
    if GetWindowRect(anchor, &mut anchor_rect) == 0 {
        return;
    }
    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let screen_h = GetSystemMetrics(SM_CYSCREEN);
    let (x, y) = selection_popup_origin(anchor_rect, screen_w, screen_h, popup_w, popup_h);
    SetWindowPos(popup, HWND_TOP, x, y, popup_w, popup_h, 0);
}

fn selection_popup_origin(
    anchor_rect: Rect,
    screen_w: i32,
    screen_h: i32,
    popup_w: i32,
    popup_h: i32,
) -> (i32, i32) {
    let max_x = (screen_w - popup_w - 4).max(4);
    let x = if anchor_rect.left + popup_w > screen_w - 4 {
        (anchor_rect.right - popup_w).clamp(4, max_x)
    } else {
        anchor_rect.left.clamp(4, max_x)
    };
    let below = anchor_rect.bottom + 4;
    let y = if below + popup_h <= screen_h - 4 {
        below
    } else {
        (anchor_rect.top - popup_h - 4).max(4)
    };
    (x, y)
}

unsafe fn add_tray_icon(hwnd: Hwnd, icon: isize) {
    let mut data: NotifyIconDataW = zeroed();
    data.cb_size = size_of::<NotifyIconDataW>() as u32;
    data.hwnd = hwnd;
    data.id = TRAY_ICON_ID;
    data.flags = NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP;
    data.callback_message = WM_TRAY_ICON;
    data.icon = icon;
    copy_wide_to_fixed(APP_TITLE, &mut data.tip);
    if Shell_NotifyIconW(NIM_ADD, &mut data) != 0 {
        TRAY_ICON_ADDED.store(true, Ordering::SeqCst);
        data.timeout_or_version = NOTIFYICON_VERSION_4;
        Shell_NotifyIconW(NIM_SETVERSION, &mut data);
    }
}

unsafe fn remove_tray_icon(hwnd: Hwnd) {
    if !TRAY_ICON_ADDED.swap(false, Ordering::SeqCst) {
        return;
    }
    let mut data: NotifyIconDataW = zeroed();
    data.cb_size = size_of::<NotifyIconDataW>() as u32;
    data.hwnd = hwnd;
    data.id = TRAY_ICON_ID;
    Shell_NotifyIconW(NIM_DELETE, &mut data);
}

fn copy_wide_to_fixed<const N: usize>(text: &str, target: &mut [u16; N]) {
    let encoded: Vec<u16> = OsStr::new(text).encode_wide().collect();
    let count = encoded.len().min(N.saturating_sub(1));
    target[..count].copy_from_slice(&encoded[..count]);
    target[count] = 0;
}

unsafe fn minimize_to_tray() {
    if APP.hwnd == 0 {
        return;
    }
    DISEASE_DROPDOWN_OPEN.store(false, Ordering::SeqCst);
    SUPERIOR_PANEL_OPEN.store(false, Ordering::SeqCst);
    show_disease_dropdown(false);
    show_superior_panel(false);
    if APP.account_popup != 0 {
        ShowWindow(APP.account_popup, SW_HIDE);
    }
    ShowWindow(APP.hwnd, SW_HIDE);
    log_event("主窗口已最小化到系统托盘");
}

unsafe fn restore_main_window() {
    if APP.hwnd == 0 {
        return;
    }
    ShowWindow(APP.hwnd, SW_RESTORE);
    ShowWindow(APP.hwnd, SW_SHOW);
    if DOCK_ENABLED.load(Ordering::SeqCst) {
        dock_to_edge(true);
    }
    BringWindowToTop(APP.hwnd);
    SetForegroundWindow(APP.hwnd);
    SetFocus(APP.hwnd);
    set_status("已从系统托盘恢复。");
}

unsafe fn set_taskbar_visible(visible: bool, save: bool) {
    TASKBAR_VISIBLE.store(visible, Ordering::SeqCst);
    if APP.hwnd == 0 {
        return;
    }
    let current = GetWindowLongPtrW(APP.hwnd, GWL_EXSTYLE) as u32;
    let next = if visible {
        (current | WS_EX_APPWINDOW) & !WS_EX_TOOLWINDOW
    } else {
        (current | WS_EX_TOOLWINDOW) & !WS_EX_APPWINDOW
    };
    if current != next {
        let was_visible = IsWindowVisible(APP.hwnd) != 0;
        if was_visible {
            ShowWindow(APP.hwnd, SW_HIDE);
        }
        SetWindowLongPtrW(APP.hwnd, GWL_EXSTYLE, next as isize);
        SetWindowPos(
            APP.hwnd,
            HWND_TOP,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
        if was_visible {
            ShowWindow(APP.hwnd, SW_SHOW);
        }
    }
    if save {
        save_settings(&read_settings_from_ui());
    }
}

unsafe fn append_tray_menu_item(menu: Hmenu, id: i32, text: &str) {
    AppendMenuW(menu, MF_STRING, id as usize, wide(text).as_ptr());
}

unsafe fn check_tray_menu_item(menu: Hmenu, id: i32, checked: bool) {
    if checked {
        CheckMenuItem(menu, id as u32, MF_CHECKED);
    }
}

unsafe fn show_tray_menu() {
    if APP.hwnd == 0 {
        return;
    }
    let menu = CreatePopupMenu();
    if menu == 0 {
        return;
    }
    append_tray_menu_item(menu, ID_TRAY_SHOW, "显示窗口");
    append_tray_menu_item(menu, ID_TRAY_SETTINGS, "设置...");
    AppendMenuW(menu, MF_SEPARATOR, 0, null());
    append_tray_menu_item(menu, ID_TRAY_TASKBAR, "在任务栏显示");
    CheckMenuItem(
        menu,
        ID_TRAY_TASKBAR as u32,
        if TASKBAR_VISIBLE.load(Ordering::SeqCst) {
            MF_CHECKED
        } else {
            0
        },
    );
    AppendMenuW(menu, MF_SEPARATOR, 0, null());
    append_tray_menu_item(menu, ID_TRAY_DOCK, "启用吸附");
    check_tray_menu_item(menu, ID_TRAY_DOCK, DOCK_ENABLED.load(Ordering::SeqCst));
    append_tray_menu_item(menu, ID_TRAY_DAY, "日间模式");
    append_tray_menu_item(menu, ID_TRAY_NIGHT, "夜间模式");
    check_tray_menu_item(menu, ID_TRAY_DAY, !DARK_THEME.load(Ordering::SeqCst));
    check_tray_menu_item(menu, ID_TRAY_NIGHT, DARK_THEME.load(Ordering::SeqCst));
    append_tray_menu_item(menu, ID_TRAY_STARTUP, "开机启动");
    check_tray_menu_item(menu, ID_TRAY_STARTUP, is_startup_enabled());
    AppendMenuW(menu, MF_SEPARATOR, 0, null());
    append_tray_menu_item(menu, ID_TRAY_EXIT, "退出");

    let mut cursor = Point { x: 0, y: 0 };
    GetCursorPos(&mut cursor);
    SetForegroundWindow(APP.hwnd);
    let command = TrackPopupMenu(
        menu,
        TPM_RIGHTBUTTON | TPM_RETURNCMD,
        cursor.x,
        cursor.y,
        0,
        APP.hwnd,
        null(),
    );
    DestroyMenu(menu);

    match command {
        ID_TRAY_SHOW => restore_main_window(),
        ID_TRAY_SETTINGS => {
            show_settings_window();
        }
        ID_TRAY_TASKBAR => {
            let visible = !TASKBAR_VISIBLE.load(Ordering::SeqCst);
            set_taskbar_visible(visible, true);
            set_status(if visible {
                "已允许主窗口显示在任务栏。"
            } else {
                "已隐藏主窗口的任务栏按钮。"
            });
        }
        ID_TRAY_DOCK => {
            let enabled = !DOCK_ENABLED.load(Ordering::SeqCst);
            set_dock_enabled(enabled);
            save_settings(&read_settings_from_ui());
        }
        ID_TRAY_DAY => {
            set_theme(false);
            save_settings(&read_settings_from_ui());
        }
        ID_TRAY_NIGHT => {
            set_theme(true);
            save_settings(&read_settings_from_ui());
        }
        ID_TRAY_STARTUP => {
            let enabled = !is_startup_enabled();
            if set_startup_enabled(enabled) {
                set_checkbox(ID_STARTUP_CHECK, APP.startup_check, enabled);
                save_settings(&read_settings_from_ui());
            } else {
                set_status("开机自启动设置失败。");
            }
        }
        ID_TRAY_EXIT => {
            PostMessageW(APP.hwnd, WM_CLOSE, 0, 0);
        }
        _ => {}
    }
}

unsafe fn api_doc_index_from_hwnd(hwnd: Hwnd) -> Option<usize> {
    (0..API_DOC_COUNT).find(|&index| APP.api_doc_rows[index] == hwnd)
}

unsafe fn show_api_doc_context_menu(index: usize) {
    let menu = CreatePopupMenu();
    if menu == 0 {
        return;
    }
    append_tray_menu_item(menu, ID_API_CONTEXT_WRITEBACK, "写回病历系统");
    let mut cursor = Point { x: 0, y: 0 };
    GetCursorPos(&mut cursor);
    SetForegroundWindow(APP.hwnd);
    let command = TrackPopupMenu(
        menu,
        TPM_RIGHTBUTTON | TPM_RETURNCMD,
        cursor.x,
        cursor.y,
        0,
        APP.hwnd,
        null(),
    );
    DestroyMenu(menu);
    if command == ID_API_CONTEXT_WRITEBACK {
        writeback_api_doc(index);
    }
}

unsafe fn dock_to_edge(visible: bool) {
    if APP.hwnd == 0 {
        return;
    }
    let mut rect = Rect::default();
    let (x, y) = dock_target_position(
        visible,
        if GetWindowRect(APP.hwnd, &mut rect) != 0 {
            Some(rect)
        } else {
            None
        },
    );
    SetWindowPos(
        APP.hwnd,
        HWND_TOP,
        x,
        y,
        DEFAULT_WINDOW_W,
        DEFAULT_WINDOW_H,
        0,
    );
    update_main_dock_region(visible, DEFAULT_WINDOW_H);
    DOCK_VISIBLE.store(visible, Ordering::SeqCst);
    DOCK_TARGET_VISIBLE.store(visible, Ordering::SeqCst);
    update_dock_button();
}

unsafe fn update_dock_state() {
    if !DOCK_ENABLED.load(Ordering::SeqCst) {
        hide_dock_handle_window();
        return;
    }
    sync_dock_to_current_display();
    if APP.hwnd == 0 || selection_popup_is_open() {
        return;
    }
    if DOCK_HANDLE_HWND != 0 && IsWindowVisible(DOCK_HANDLE_HWND) != 0 {
        return;
    }
    if IsWindowVisible(APP.hwnd) == 0 {
        return;
    }
    if SETTINGS_DIALOG.hwnd != 0 && IsWindowVisible(SETTINGS_DIALOG.hwnd) != 0 {
        DOCK_HIDE_DEADLINE_MS.store(0, Ordering::SeqCst);
        DOCK_VISIBLE.store(true, Ordering::SeqCst);
        DOCK_TARGET_VISIBLE.store(true, Ordering::SeqCst);
        animate_dock_step(true);
        return;
    }
    if current_epoch_millis() < DOCK_SUSPEND_UNTIL_MS.load(Ordering::SeqCst) {
        DOCK_VISIBLE.store(true, Ordering::SeqCst);
        DOCK_TARGET_VISIBLE.store(true, Ordering::SeqCst);
        animate_dock_step(true);
        return;
    }
    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let mut cursor = Point { x: 0, y: 0 };
    if GetCursorPos(&mut cursor) == 0 {
        return;
    }
    let mut rect = Rect::default();
    if GetWindowRect(APP.hwnd, &mut rect) == 0 {
        return;
    }
    update_dock_edge_from_position(rect);
    let hidden_or_hiding = !DOCK_TARGET_VISIBLE.load(Ordering::SeqCst);
    let expanded_keep_zone = DOCK_VISIBLE.load(Ordering::SeqCst)
        && cursor.x >= rect.left
        && cursor.x <= (rect.right + DOCK_PEEK_W).min(screen_w)
        && cursor.y >= rect.top
        && cursor.y <= rect.bottom;
    let inside_window = (!hidden_or_hiding
        && cursor.x >= rect.left
        && cursor.x <= rect.right
        && cursor.y >= rect.top
        && cursor.y <= rect.bottom)
        || expanded_keep_zone;
    let edge = DOCK_EDGE.load(Ordering::SeqCst);
    let cursor_wants_show = inside_window || cursor_in_dock_hot_zone(edge, cursor, rect, screen_w);
    let should_show = if cursor_wants_show {
        DOCK_HIDE_DEADLINE_MS.store(0, Ordering::SeqCst);
        true
    } else {
        let delay = DOCK_HIDE_DELAY_MS.load(Ordering::SeqCst);
        if delay == 0 {
            DOCK_HIDE_DEADLINE_MS.store(0, Ordering::SeqCst);
            false
        } else {
            let now = current_epoch_millis();
            let deadline = DOCK_HIDE_DEADLINE_MS.load(Ordering::SeqCst);
            if deadline == 0 {
                DOCK_HIDE_DEADLINE_MS.store(now.saturating_add(delay), Ordering::SeqCst);
                true
            } else {
                now < deadline
            }
        }
    };
    if !should_show && edge == DOCK_EDGE_RIGHT {
        hide_main_to_dock_handle();
        return;
    }
    DOCK_TARGET_VISIBLE.store(should_show, Ordering::SeqCst);
    animate_dock_step(should_show);
}

unsafe fn sync_dock_to_current_display() {
    let screen_w = GetSystemMetrics(SM_CXSCREEN).max(1);
    let screen_h = GetSystemMetrics(SM_CYSCREEN).max(1);
    let previous_w = DOCK_LAST_SCREEN_W.swap(screen_w, Ordering::SeqCst);
    let previous_h = DOCK_LAST_SCREEN_H.swap(screen_h, Ordering::SeqCst);
    if previous_w == 0 || previous_h == 0 || (previous_w == screen_w && previous_h == screen_h) {
        return;
    }

    DOCK_HIDE_DEADLINE_MS.store(0, Ordering::SeqCst);
    log_event(&format!(
        "检测到显示器尺寸变化：{}x{} -> {}x{}，已重新计算右侧吸附位置",
        previous_w, previous_h, screen_w, screen_h
    ));

    if DOCK_HANDLE_HWND != 0 && IsWindowVisible(DOCK_HANDLE_HWND) != 0 {
        show_dock_handle_window();
        return;
    }
    if APP.hwnd == 0 {
        return;
    }
    let mut rect = Rect::default();
    let (x, y) = dock_target_position(
        DOCK_TARGET_VISIBLE.load(Ordering::SeqCst),
        if GetWindowRect(APP.hwnd, &mut rect) != 0 {
            Some(rect)
        } else {
            None
        },
    );
    SetWindowPos(
        APP.hwnd,
        HWND_TOP,
        x,
        y,
        DEFAULT_WINDOW_W,
        DEFAULT_WINDOW_H,
        0,
    );
}

fn cleanup_old_executables() {
    let Ok(root) = std::env::current_dir() else {
        return;
    };
    for file in [
        "resident_typer_update.exe",
        "resident_typer_update_pending.exe",
        "resident_typer_v1.3.28.exe",
        "resident_typer_v1.3.29.exe",
        "resident_typer_v1.3.30.exe",
        "resident_typer_v1.3.31.exe",
        "resident_typer_v1.3.32.exe",
        "resident_typer_v1.3.33.exe",
        "resident_typer_v1.3.34.exe",
        "resident_typer_v1.3.35.exe",
    ] {
        let _ = fs::remove_file(root.join(file));
    }
}

unsafe fn close_previous_instance() {
    let class_name = wide(APP_CLASS);
    let previous = FindWindowW(class_name.as_ptr(), null());
    if previous == 0 {
        return;
    }

    PostMessageW(previous, WM_CLOSE, 0, 0);
    for _ in 0..20 {
        if IsWindow(previous) == 0 {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn cursor_in_dock_hot_zone(edge: i32, cursor: Point, window: Rect, screen_w: i32) -> bool {
    match edge {
        DOCK_EDGE_TOP => {
            cursor.y <= DOCK_HOT_ZONE && cursor.x >= window.left && cursor.x <= window.right
        }
        _ => {
            // Only the visible handle span can wake a hidden right-docked window.
            // Once expanded, update_dock_state separately keeps the window body open.
            let tail_h = DOCK_TAIL_H.min((window.bottom - window.top).max(1));
            let tail_top = window.top + ((window.bottom - window.top - tail_h).max(0) / 2);
            cursor.x >= screen_w - DOCK_HOT_ZONE
                && cursor.y >= tail_top
                && cursor.y <= tail_top + tail_h
        }
    }
}

unsafe fn ensure_dock_handle_window() -> Hwnd {
    if DOCK_HANDLE_HWND != 0 && IsWindow(DOCK_HANDLE_HWND) != 0 {
        return DOCK_HANDLE_HWND;
    }
    DOCK_HANDLE_HWND = CreateWindowExW(
        WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
        wide(DOCK_HANDLE_CLASS).as_ptr(),
        wide("").as_ptr(),
        WS_POPUP,
        0,
        0,
        DOCK_HANDLE_W,
        DOCK_HANDLE_H,
        0,
        0,
        GetModuleHandleW(null()),
        null_mut(),
    );
    DOCK_HANDLE_HWND
}

unsafe fn show_dock_handle_window() {
    let hwnd = ensure_dock_handle_window();
    if hwnd == 0 {
        return;
    }
    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let screen_h = GetSystemMetrics(SM_CYSCREEN);
    MoveWindow(
        hwnd,
        (screen_w - DOCK_HANDLE_W).max(0),
        ((screen_h - DOCK_HANDLE_H) / 2).max(0),
        DOCK_HANDLE_W,
        DOCK_HANDLE_H,
        1,
    );
    ShowWindow(hwnd, SW_SHOW);
    BringWindowToTop(hwnd);
}

unsafe fn hide_dock_handle_window() {
    if DOCK_HANDLE_HWND != 0 && IsWindow(DOCK_HANDLE_HWND) != 0 {
        ShowWindow(DOCK_HANDLE_HWND, SW_HIDE);
    }
}

unsafe fn hide_main_to_dock_handle() {
    show_disease_dropdown(false);
    show_superior_panel(false);
    hide_dock_handle_window();
    ShowWindow(APP.hwnd, SW_HIDE);
    DOCK_VISIBLE.store(false, Ordering::SeqCst);
    DOCK_TARGET_VISIBLE.store(false, Ordering::SeqCst);
    show_dock_handle_window();
}

unsafe fn restore_main_from_dock_handle() {
    hide_dock_handle_window();
    DOCK_VISIBLE.store(true, Ordering::SeqCst);
    DOCK_TARGET_VISIBLE.store(true, Ordering::SeqCst);
    SetWindowRgn(APP.hwnd, 0, 1);
    set_dock_hidden_chrome(false);
    set_main_controls_visible(true);
    dock_to_edge(true);
    ShowWindow(APP.hwnd, SW_SHOW);
    InvalidateRect(APP.hwnd, null(), 1);
}

fn current_epoch_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn suspend_docking_after_selection() {
    const PAUSE_MS: u64 = 4_000;
    DOCK_SUSPEND_UNTIL_MS.store(
        current_epoch_millis().saturating_add(PAUSE_MS),
        Ordering::SeqCst,
    );
    DOCK_VISIBLE.store(true, Ordering::SeqCst);
    DOCK_TARGET_VISIBLE.store(true, Ordering::SeqCst);
}

fn selection_popup_is_open() -> bool {
    DISEASE_DROPDOWN_OPEN.load(Ordering::SeqCst)
        || SUPERIOR_PANEL_OPEN.load(Ordering::SeqCst)
        || unsafe { APP.account_popup != 0 && IsWindowVisible(APP.account_popup) != 0 }
}

unsafe fn animate_dock_step(show: bool) -> bool {
    let mut rect = Rect::default();
    if GetWindowRect(APP.hwnd, &mut rect) == 0 {
        return false;
    }
    let (target_x, target_y) = dock_target_position(show, Some(rect));
    let current_x = rect.left;
    let current_y = rect.top;
    if current_x == target_x && current_y == target_y {
        DOCK_VISIBLE.store(show, Ordering::SeqCst);
        return true;
    }
    let next_x = smooth_step(current_x, target_x);
    let next_y = smooth_step(current_y, target_y);
    SetWindowPos(
        APP.hwnd,
        HWND_TOP,
        next_x,
        next_y,
        DEFAULT_WINDOW_W,
        rect.bottom - rect.top,
        0,
    );
    // Keep the normal window chrome during the slide. The independent handle
    // replaces the main window only after it has fully moved off-screen.
    update_main_dock_region(true, rect.bottom - rect.top);
    false
}

unsafe fn update_dock_edge_from_position(rect: Rect) {
    if !DOCK_TARGET_VISIBLE.load(Ordering::SeqCst) {
        return;
    }
    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    if rect.top <= 24 {
        DOCK_EDGE.store(DOCK_EDGE_TOP, Ordering::SeqCst);
    } else if screen_w - rect.right <= 24 {
        DOCK_EDGE.store(DOCK_EDGE_RIGHT, Ordering::SeqCst);
    }
}

unsafe fn dock_target_position(visible: bool, current: Option<Rect>) -> (i32, i32) {
    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let screen_h = GetSystemMetrics(SM_CYSCREEN);
    let edge = DOCK_EDGE.load(Ordering::SeqCst);
    match edge {
        DOCK_EDGE_TOP => {
            let x = current
                .map(|r| r.left)
                .unwrap_or(DEFAULT_WINDOW_X)
                .clamp(0, (screen_w - DEFAULT_WINDOW_W).max(0));
            let y = if visible {
                0
            } else {
                -(DEFAULT_WINDOW_H - DOCK_PEEK_W)
            };
            (x, y)
        }
        _ => {
            let x = if visible {
                screen_w - DEFAULT_WINDOW_W
            } else {
                screen_w
            };
            let y = current
                .map(|r| r.top)
                .unwrap_or(DEFAULT_WINDOW_Y)
                .clamp(0, (screen_h - DEFAULT_WINDOW_H).max(0));
            (x.max(0), y)
        }
    }
}

unsafe fn update_main_dock_region(show: bool, height: i32) {
    if APP.hwnd == 0 {
        return;
    }
    let edge = DOCK_EDGE.load(Ordering::SeqCst);
    if show || edge != DOCK_EDGE_RIGHT {
        set_dock_hidden_chrome(false);
        if DOCK_REGION_HIDDEN.swap(false, Ordering::SeqCst) {
            SetWindowRgn(APP.hwnd, 0, 1);
            set_main_controls_visible(true);
            InvalidateRect(APP.hwnd, null(), 1);
            UpdateWindow(APP.hwnd);
        }
        return;
    }
    let tail_h = DOCK_TAIL_H.min(height.max(1));
    let tail_top = ((height - tail_h).max(0)) / 2;
    let region = CreateRoundRectRgn(0, tail_top, DOCK_PEEK_W + 1, tail_top + tail_h + 1, 18, 18);
    if region != 0 {
        // Set the state before applying the region: Windows may synchronously
        // repaint while SetWindowRgn is running.
        if !DOCK_REGION_HIDDEN.swap(true, Ordering::SeqCst) {
            set_main_controls_visible(false);
        }
        set_dock_hidden_chrome(true);
        SetWindowRgn(APP.hwnd, region, 1);
        InvalidateRect(APP.hwnd, null(), 1);
    }
}

unsafe fn register_dock_handle_class(h_instance: Hinstance) {
    let class_name = wide(DOCK_HANDLE_CLASS);
    let wc = WndClassW {
        style: 0,
        lpfn_wnd_proc: dock_handle_proc,
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance,
        h_icon: 0,
        h_cursor: LoadCursorW(0, IDC_ARROW),
        hbr_background: 0,
        lpsz_menu_name: null(),
        lpsz_class_name: class_name.as_ptr(),
    };
    RegisterClassW(&wc);
}

extern "system" fn dock_handle_proc(
    hwnd: Hwnd,
    msg: u32,
    _w_param: Wparam,
    _l_param: Lparam,
) -> Lresult {
    unsafe {
        match msg {
            WM_PAINT => {
                let mut paint = PaintStruct {
                    hdc: 0,
                    erase: 0,
                    rc_paint: Rect::default(),
                    restore: 0,
                    inc_update: 0,
                    reserved: [0; 32],
                };
                let hdc = BeginPaint(hwnd, &mut paint);
                if hdc != 0 {
                    draw_dock_handle(hdc, hwnd);
                }
                EndPaint(hwnd, &paint);
                0
            }
            WM_ERASEBKGND => 1,
            WM_LBUTTONDOWN => {
                restore_main_from_dock_handle();
                0
            }
            WM_DESTROY => {
                DOCK_HANDLE_HWND = 0;
                0
            }
            _ => DefWindowProcW(hwnd, msg, _w_param, _l_param),
        }
    }
}

unsafe fn set_main_controls_visible(visible: bool) {
    let mode = if visible { SW_SHOW } else { SW_HIDE };
    for hwnd in [
        APP.logo,
        APP.header,
        APP.divider,
        APP.status,
        APP.version,
        APP.tab_templates,
        APP.tab_shortcuts,
        APP.tab_api,
        APP.settings_button,
        APP.cleanup_button,
        APP.theme_toggle,
        APP.dock_toggle,
    ] {
        if hwnd != 0 {
            ShowWindow(hwnd, mode);
        }
    }
    for i in 0..TEMPLATE_CONTROL_COUNT {
        if TEMPLATE_CONTROLS[i] != 0 {
            ShowWindow(
                TEMPLATE_CONTROLS[i],
                if visible && TEMPLATE_PAGE_VISIBLE.load(Ordering::SeqCst) {
                    SW_SHOW
                } else {
                    SW_HIDE
                },
            );
        }
    }
    for i in 0..SHORTCUT_CONTROL_COUNT {
        if SHORTCUT_CONTROLS[i] != 0 {
            ShowWindow(
                SHORTCUT_CONTROLS[i],
                if visible
                    && !TEMPLATE_PAGE_VISIBLE.load(Ordering::SeqCst)
                    && !API_PAGE_VISIBLE.load(Ordering::SeqCst)
                {
                    SW_SHOW
                } else {
                    SW_HIDE
                },
            );
        }
    }
    for i in 0..API_CONTROL_COUNT {
        if API_CONTROLS[i] != 0 {
            ShowWindow(
                API_CONTROLS[i],
                if visible && API_PAGE_VISIBLE.load(Ordering::SeqCst) {
                    SW_SHOW
                } else {
                    SW_HIDE
                },
            );
        }
    }
    if !visible {
        show_disease_dropdown(false);
        show_superior_panel(false);
    }
}

unsafe fn set_dock_hidden_chrome(hidden: bool) {
    if APP.hwnd == 0 || DOCK_CHROMELESS_HIDDEN.swap(hidden, Ordering::SeqCst) == hidden {
        return;
    }
    let style = if hidden {
        WS_POPUP | WS_VISIBLE
    } else {
        WS_OVERLAPPEDWINDOW | WS_VISIBLE
    };
    SetWindowLongPtrW(APP.hwnd, GWL_STYLE, style as isize);
    SetWindowPos(
        APP.hwnd,
        HWND_TOP,
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED,
    );
}

unsafe fn draw_dock_handle(hdc: Hdc, hwnd: Hwnd) {
    let mut rect = Rect::default();
    GetClientRect(hwnd, &mut rect);
    let top = 3;
    let bottom = (rect.bottom - 3).max(top + 1);
    let blue = CreateSolidBrush(rgb(0, 102, 230));
    let cyan = CreatePen(PS_SOLID, 2, rgb(103, 199, 255));
    let white = CreatePen(PS_SOLID, 3, rgb(255, 255, 255));
    let old_brush = SelectObject(hdc, blue);
    let old_pen = SelectObject(hdc, cyan);
    RoundRect(hdc, 1, top, rect.right - 1, bottom, 18, 18);
    SelectObject(hdc, white);
    let center = (top + bottom) / 2;
    MoveToEx(hdc, 25, center - 11, null_mut());
    LineTo(hdc, 14, center);
    LineTo(hdc, 25, center + 11);
    SelectObject(hdc, old_pen);
    SelectObject(hdc, old_brush);
    DeleteObject(white);
    DeleteObject(cyan);
    DeleteObject(blue);
}

fn smooth_step(current: i32, target: i32) -> i32 {
    let delta = target - current;
    if delta == 0 {
        return current;
    }
    let distance = delta.abs();
    let step = ((distance + 5) / 6).clamp(DOCK_ANIM_MIN_STEP, DOCK_ANIM_MAX_STEP);
    if delta > 0 {
        (current + step).min(target)
    } else {
        (current - step).max(target)
    }
}

unsafe fn set_dock_enabled(enabled: bool) {
    DOCK_ENABLED.store(enabled, Ordering::SeqCst);
    if enabled {
        hide_dock_handle_window();
        if IsWindowVisible(APP.hwnd) == 0 {
            ShowWindow(APP.hwnd, SW_SHOW);
        }
        let mut rect = Rect::default();
        if GetWindowRect(APP.hwnd, &mut rect) != 0 {
            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            if rect.top <= screen_w - rect.right {
                DOCK_EDGE.store(DOCK_EDGE_TOP, Ordering::SeqCst);
            } else {
                DOCK_EDGE.store(DOCK_EDGE_RIGHT, Ordering::SeqCst);
            }
        }
        dock_to_edge(true);
    } else {
        hide_dock_handle_window();
        if IsWindowVisible(APP.hwnd) == 0 {
            ShowWindow(APP.hwnd, SW_SHOW);
        }
        DOCK_VISIBLE.store(true, Ordering::SeqCst);
        DOCK_TARGET_VISIBLE.store(true, Ordering::SeqCst);
        update_dock_button();
    }
}

unsafe fn update_dock_button() {
    if APP.dock_toggle != 0 {
        SetWindowTextW(
            APP.dock_toggle,
            wide(if DOCK_ENABLED.load(Ordering::SeqCst) {
                "吸附"
            } else {
                "自由"
            })
            .as_ptr(),
        );
    }
}

unsafe fn selected_disease_index() -> usize {
    if APP.disease_list == 0 {
        0
    } else {
        let index = SendMessageW(APP.disease_list, LB_GETCURSEL, 0, 0);
        if index < 0 {
            0
        } else {
            (index as usize).min(DISEASE_ITEMS.len() - 1)
        }
    }
}

unsafe fn set_selected_disease(index: usize) {
    let index = index.min(DISEASE_ITEMS.len() - 1);
    if APP.disease_list != 0 {
        SendMessageW(APP.disease_list, LB_SETCURSEL, index, 0);
    }
    update_disease_button(DISEASE_DROPDOWN_OPEN.load(Ordering::SeqCst));
}

unsafe fn update_disease_button(open: bool) {
    let index = selected_disease_index();
    let arrow = if open { "▲" } else { "▼" };
    SetWindowTextW(
        APP.disease_toggle,
        wide(&format!("疾病：{} {}", DISEASE_ITEMS[index], arrow)).as_ptr(),
    );
}

unsafe fn toggle_superior_selector(target: i32) {
    let was_open = SUPERIOR_PANEL_OPEN.load(Ordering::SeqCst);
    let same_target = SUPERIOR_TARGET.load(Ordering::SeqCst) == target;
    let show = !was_open || !same_target;
    SUPERIOR_TARGET.store(target, Ordering::SeqCst);
    SUPERIOR_PANEL_OPEN.store(show, Ordering::SeqCst);

    if show {
        DOCK_SUSPEND_UNTIL_MS.store(0, Ordering::SeqCst);
        let name = selected_superior_name(target);
        select_listbox_text(APP.superior_list, &name);
        SetWindowTextW(
            APP.superior_header,
            wide(if target == 0 {
                "选择主治上级"
            } else {
                "选择主任上级"
            })
            .as_ptr(),
        );
    }
    set_superior_button_text(0, &selected_superior_name(0), show && target == 0);
    set_superior_button_text(1, &selected_superior_name(1), show && target == 1);
    show_superior_panel(show);
    set_status(if show {
        "已展开上级医师目录；单击姓名后自动收起。"
    } else {
        "已收起上级医师目录。"
    });
}

unsafe fn set_superior_button_text(target: i32, name: &str, open: bool) {
    let control = if target == 0 {
        APP.attending_superior
    } else {
        APP.chief_superior
    };
    let prefix = if target == 0 { "主治" } else { "主任" };
    let arrow = if open { "▲" } else { "▼" };
    SetWindowTextW(
        control,
        wide(&format!("{}：{} {}", prefix, name, arrow)).as_ptr(),
    );
    InvalidateRect(control, null(), 1);
}

unsafe fn selected_superior_name(target: i32) -> String {
    let control = if target == 0 {
        APP.attending_superior
    } else {
        APP.chief_superior
    };
    let text = get_window_text(control);
    text.split_once('：')
        .map(|(_, value)| value.trim().trim_end_matches(['▼', '▲']).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "裴傲".to_string())
}

unsafe fn selected_listbox_text(listbox: Hwnd) -> Option<String> {
    let index = SendMessageW(listbox, LB_GETCURSEL, 0, 0);
    if index < 0 {
        return None;
    }
    let len = SendMessageW(listbox, LB_GETTEXTLEN, index as Wparam, 0);
    if len < 0 {
        return None;
    }
    let mut buffer = vec![0u16; len as usize + 1];
    SendMessageW(
        listbox,
        LB_GETTEXT,
        index as Wparam,
        buffer.as_mut_ptr() as Lparam,
    );
    let value = String::from_utf16_lossy(&buffer[..len as usize]);
    (!value.trim().is_empty()).then_some(value)
}

unsafe fn select_listbox_text(listbox: Hwnd, value: &str) {
    let value = wide(value);
    let index = SendMessageW(
        listbox,
        LB_FINDSTRINGEXACT,
        Wparam::MAX,
        value.as_ptr() as Lparam,
    );
    if index >= 0 {
        SendMessageW(listbox, LB_SETCURSEL, index as Wparam, 0);
    }
}

unsafe fn selected_ward_building_index() -> usize {
    let text = get_window_text(APP.ward_building);
    WARD_BUILDINGS
        .iter()
        .position(|building| text.contains(building))
        .unwrap_or(0)
}

unsafe fn selected_ward_floor() -> u32 {
    let text = get_window_text(APP.ward_floor);
    let digits: String = text.chars().filter(|ch| ch.is_ascii_digit()).collect();
    digits.parse::<u32>().unwrap_or(5).clamp(2, 10)
}

unsafe fn set_ward_selection(building_index: usize, floor: u32) {
    let building_index = if building_index >= 2 {
        building_index - 2
    } else {
        building_index
    };
    let building_index = building_index.min(WARD_BUILDINGS.len() - 1);
    let floor = floor.clamp(2, 10);
    if APP.ward_building != 0 {
        SetWindowTextW(
            APP.ward_building,
            wide(&format!("楼栋：{}", WARD_BUILDINGS[building_index])).as_ptr(),
        );
    }
    if APP.ward_floor != 0 {
        SetWindowTextW(APP.ward_floor, wide(&format!("{}楼", floor)).as_ptr());
    }
}

unsafe fn generated_nursing_account() -> String {
    let building_index = selected_ward_building_index();
    let floor = selected_ward_floor() as i32;
    let offset = match WARD_BUILDINGS[building_index] {
        "C" => -3,
        "D" => -2,
        _ => -3,
    };
    let value = ((floor * 2 + offset).max(0) * 100) as u32;
    format!("b{:04}", value)
}

fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

fn theme_bg_color() -> u32 {
    if DARK_THEME.load(Ordering::SeqCst) {
        rgb(28, 28, 30)
    } else {
        rgb(242, 242, 247)
    }
}

fn theme_text_color() -> u32 {
    if DARK_THEME.load(Ordering::SeqCst) {
        rgb(245, 245, 247)
    } else {
        rgb(29, 29, 31)
    }
}

fn theme_secondary_text_color() -> u32 {
    if DARK_THEME.load(Ordering::SeqCst) {
        rgb(174, 174, 178)
    } else {
        rgb(110, 110, 115)
    }
}

fn theme_control_color() -> u32 {
    if DARK_THEME.load(Ordering::SeqCst) {
        rgb(38, 40, 44)
    } else {
        rgb(246, 249, 253)
    }
}

fn theme_popup_bg_color() -> u32 {
    if DARK_THEME.load(Ordering::SeqCst) {
        rgb(32, 43, 56)
    } else {
        rgb(242, 248, 255)
    }
}

unsafe fn theme_brush() -> Hbrush {
    if DARK_THEME.load(Ordering::SeqCst) {
        NIGHT_BRUSH
    } else {
        DAY_BRUSH
    }
}

unsafe fn theme_control_brush() -> Hbrush {
    if DARK_THEME.load(Ordering::SeqCst) {
        NIGHT_CONTROL_BRUSH
    } else {
        DAY_CONTROL_BRUSH
    }
}

unsafe fn theme_popup_brush() -> Hbrush {
    if DARK_THEME.load(Ordering::SeqCst) {
        NIGHT_POPUP_BRUSH
    } else {
        DAY_POPUP_BRUSH
    }
}

unsafe fn apply_theme_to_dc(hdc: Hdc) {
    SetTextColor(hdc, theme_text_color());
    SetBkColor(hdc, theme_bg_color());
}

unsafe fn apply_control_theme_to_dc(hdc: Hdc) {
    SetTextColor(hdc, theme_text_color());
    SetBkColor(hdc, theme_control_color());
}

unsafe fn apply_popup_theme_to_dc(hdc: Hdc) {
    SetTextColor(hdc, theme_text_color());
    SetBkColor(
        hdc,
        if DARK_THEME.load(Ordering::SeqCst) {
            rgb(32, 43, 56)
        } else {
            rgb(242, 248, 255)
        },
    );
}

unsafe fn apply_static_theme_to_dc(hdc: Hdc, control: Hwnd) {
    let secondary = control == APP.version || control == APP.divider;
    SetTextColor(
        hdc,
        if secondary {
            theme_secondary_text_color()
        } else {
            theme_text_color()
        },
    );
    SetBkColor(hdc, theme_bg_color());
}

unsafe fn apply_window_chrome(dark: bool) {
    if APP.hwnd == 0 {
        return;
    }
    let dark_value: i32 = if dark { 1 } else { 0 };
    let corner = DWMWCP_ROUND;
    let caption = if dark {
        rgb(44, 44, 46)
    } else {
        rgb(242, 242, 247)
    };
    let border = if dark {
        rgb(72, 72, 74)
    } else {
        rgb(209, 209, 214)
    };
    let text = if dark {
        rgb(245, 245, 247)
    } else {
        rgb(29, 29, 31)
    };
    let int_size = size_of::<i32>() as u32;
    let color_size = size_of::<u32>() as u32;
    DwmSetWindowAttribute(
        APP.hwnd,
        DWMWA_USE_IMMERSIVE_DARK_MODE,
        &dark_value as *const _ as *const _,
        int_size,
    );
    DwmSetWindowAttribute(
        APP.hwnd,
        DWMWA_WINDOW_CORNER_PREFERENCE,
        &corner as *const _ as *const _,
        int_size,
    );
    DwmSetWindowAttribute(
        APP.hwnd,
        DWMWA_CAPTION_COLOR,
        &caption as *const _ as *const _,
        color_size,
    );
    DwmSetWindowAttribute(
        APP.hwnd,
        DWMWA_BORDER_COLOR,
        &border as *const _ as *const _,
        color_size,
    );
    DwmSetWindowAttribute(
        APP.hwnd,
        DWMWA_TEXT_COLOR,
        &text as *const _ as *const _,
        color_size,
    );
}

unsafe fn apply_settings_window_chrome(hwnd: Hwnd, dark: bool) {
    if hwnd == 0 {
        return;
    }
    let dark_value: i32 = if dark { 1 } else { 0 };
    let corner = DWMWCP_ROUND;
    let caption = if dark {
        rgb(44, 44, 46)
    } else {
        rgb(242, 242, 247)
    };
    let border = if dark {
        rgb(72, 72, 74)
    } else {
        rgb(209, 209, 214)
    };
    let text = if dark {
        rgb(245, 245, 247)
    } else {
        rgb(29, 29, 31)
    };
    let int_size = size_of::<i32>() as u32;
    let color_size = size_of::<u32>() as u32;
    DwmSetWindowAttribute(
        hwnd,
        DWMWA_USE_IMMERSIVE_DARK_MODE,
        &dark_value as *const _ as *const _,
        int_size,
    );
    DwmSetWindowAttribute(
        hwnd,
        DWMWA_WINDOW_CORNER_PREFERENCE,
        &corner as *const _ as *const _,
        int_size,
    );
    DwmSetWindowAttribute(
        hwnd,
        DWMWA_CAPTION_COLOR,
        &caption as *const _ as *const _,
        color_size,
    );
    DwmSetWindowAttribute(
        hwnd,
        DWMWA_BORDER_COLOR,
        &border as *const _ as *const _,
        color_size,
    );
    DwmSetWindowAttribute(
        hwnd,
        DWMWA_TEXT_COLOR,
        &text as *const _ as *const _,
        color_size,
    );
}

unsafe fn set_theme(dark: bool) {
    DARK_THEME.store(dark, Ordering::SeqCst);
    if APP.theme_toggle != 0 {
        SetWindowTextW(
            APP.theme_toggle,
            wide(if dark { "日间" } else { "夜间" }).as_ptr(),
        );
    }
    if APP.hwnd != 0 {
        apply_window_chrome(dark);
        SetClassLongPtrW(APP.hwnd, GCLP_HBRBACKGROUND, theme_brush());
        InvalidateRect(APP.hwnd, null(), 1);
    }
    if SETTINGS_DIALOG.hwnd != 0 {
        apply_settings_window_chrome(SETTINGS_DIALOG.hwnd, dark);
        SetClassLongPtrW(SETTINGS_DIALOG.hwnd, GCLP_HBRBACKGROUND, theme_brush());
        InvalidateRect(SETTINGS_DIALOG.hwnd, null(), 1);
    }
    for popup in [APP.disease_popup, APP.superior_popup, APP.account_popup] {
        if popup != 0 {
            InvalidateRect(popup, null(), 1);
        }
    }
    for list in [APP.disease_list, APP.superior_list] {
        if list != 0 {
            InvalidateRect(list, null(), 1);
        }
    }
}

unsafe fn draw_owner_button(draw: *const DrawItemStruct) {
    if draw.is_null() {
        return;
    }
    let item = &*draw;
    let id = item.ctl_id as i32;
    let dark = DARK_THEME.load(Ordering::SeqCst);
    let pressed = item.item_state & ODS_SELECTED != 0;
    let focused = item.item_state & ODS_FOCUS != 0;
    let is_tab = id == ID_TAB_TEMPLATES || id == ID_TAB_SHORTCUTS || id == ID_TAB_API;
    let checked = if is_checkbox_id(id) {
        checkbox_checked(id)
    } else if id == ID_TAB_TEMPLATES {
        TEMPLATE_PAGE_VISIBLE.load(Ordering::SeqCst)
    } else if id == ID_TAB_SHORTCUTS {
        !TEMPLATE_PAGE_VISIBLE.load(Ordering::SeqCst) && !API_PAGE_VISIBLE.load(Ordering::SeqCst)
    } else if id == ID_TAB_API {
        API_PAGE_VISIBLE.load(Ordering::SeqCst)
    } else if id == ID_SETTINGS_API_MODE {
        SETTINGS_API_MODE_CHECKED.load(Ordering::SeqCst)
    } else {
        item.item_state & ODS_CHECKED != 0
            || SendMessageW(item.hwnd_item, BM_GETCHECK, 0, 0) == BST_CHECKED
    };
    let is_stop = id == ID_STOP;
    let is_toolbar = id == ID_DOCK_TOGGLE || id == ID_THEME_TOGGLE;
    let is_resolution_option = is_resolution_option_id(id);
    let is_feature_toggle =
        id == ID_CLIPBOARD_AUTO || id == ID_SHORTCUT_SAVE_ORDER || is_resolution_option;
    if is_checkbox_id(id) && !is_feature_toggle {
        draw_owner_checkbox(item, checked, focused);
        return;
    }

    let (bg, border, text_color) = if is_feature_toggle && checked {
        if dark {
            (rgb(24, 65, 101), rgb(10, 132, 255), rgb(245, 245, 247))
        } else {
            (rgb(214, 236, 255), rgb(0, 110, 230), rgb(0, 83, 176))
        }
    } else if dark {
        if is_tab && checked {
            (rgb(72, 72, 74), rgb(99, 99, 102), rgb(10, 132, 255))
        } else if is_tab {
            (rgb(44, 44, 46), rgb(72, 72, 74), rgb(174, 174, 178))
        } else if is_stop {
            (
                if pressed {
                    rgb(108, 38, 42)
                } else {
                    rgb(73, 38, 41)
                },
                rgb(126, 54, 59),
                rgb(255, 105, 97),
            )
        } else if pressed {
            (rgb(64, 64, 67), rgb(99, 99, 102), rgb(245, 245, 247))
        } else if is_toolbar {
            (rgb(58, 58, 60), rgb(72, 72, 74), rgb(209, 209, 214))
        } else {
            (rgb(44, 44, 46), rgb(72, 72, 74), rgb(245, 245, 247))
        }
    } else if is_tab && checked {
        (rgb(255, 255, 255), rgb(209, 209, 214), rgb(0, 122, 255))
    } else if is_tab {
        (rgb(229, 229, 234), rgb(209, 209, 214), rgb(110, 110, 115))
    } else if is_stop {
        (
            if pressed {
                rgb(255, 214, 217)
            } else {
                rgb(255, 236, 238)
            },
            rgb(255, 184, 189),
            rgb(215, 0, 21),
        )
    } else if pressed {
        (rgb(224, 239, 255), rgb(133, 190, 255), rgb(0, 98, 204))
    } else if is_toolbar {
        (rgb(229, 229, 234), rgb(209, 209, 214), rgb(58, 58, 60))
    } else {
        (rgb(255, 255, 255), rgb(209, 209, 214), rgb(29, 29, 31))
    };

    let canvas_brush = CreateSolidBrush(theme_bg_color());
    FillRect(item.hdc, &item.rc_item, canvas_brush);
    DeleteObject(canvas_brush);

    let old_pen = SelectObject(item.hdc, GetStockObject(NULL_PEN));
    let outer_brush = CreateSolidBrush(if focused { rgb(0, 122, 255) } else { border });
    let old_brush = SelectObject(item.hdc, outer_brush);
    let radius = if is_tab || is_toolbar { 14 } else { 12 };
    RoundRect(
        item.hdc,
        item.rc_item.left,
        item.rc_item.top,
        item.rc_item.right,
        item.rc_item.bottom,
        radius,
        radius,
    );

    let inset = if is_feature_toggle && checked {
        3
    } else if focused {
        2
    } else {
        1
    };
    let inner_brush = CreateSolidBrush(bg);
    SelectObject(item.hdc, inner_brush);
    RoundRect(
        item.hdc,
        item.rc_item.left + inset,
        item.rc_item.top + inset,
        item.rc_item.right - inset,
        item.rc_item.bottom - inset,
        (radius - 2).max(4),
        (radius - 2).max(4),
    );
    SelectObject(item.hdc, old_brush);
    SelectObject(item.hdc, old_pen);
    DeleteObject(inner_brush);
    DeleteObject(outer_brush);

    SetBkMode(item.hdc, TRANSPARENT);
    SetTextColor(item.hdc, text_color);
    let raw_text = get_window_text(item.hwnd_item);
    if matches!(
        id,
        ID_SHORTCUT_SAVE_ORDER | ID_CLIPBOARD_AUTO | ID_SHORTCUT_VT | ID_SHORTCUT_CLINICAL_CONTINUE
    ) {
        let (title, shortcut) = raw_text.split_once('\n').unwrap_or((&raw_text, ""));
        let offset = if pressed { 1 } else { 0 };
        let mut title_rect = item.rc_item;
        title_rect.top += 3 + offset;
        title_rect.bottom = title_rect.top + 23;
        let old_font = SelectObject(item.hdc, BUTTON_FONT);
        let title_text = wide(title);
        DrawTextW(
            item.hdc,
            title_text.as_ptr(),
            -1,
            &mut title_rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );

        SetTextColor(item.hdc, theme_secondary_text_color());
        SelectObject(item.hdc, NOTE_FONT);
        let mut shortcut_rect = item.rc_item;
        shortcut_rect.top += 23 + offset;
        shortcut_rect.bottom -= 3;
        let shortcut_text = wide(shortcut);
        DrawTextW(
            item.hdc,
            shortcut_text.as_ptr(),
            -1,
            &mut shortcut_rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
        SelectObject(item.hdc, old_font);
    } else {
        let old_font = SelectObject(item.hdc, BUTTON_FONT);
        let text = wide(&raw_text);
        let mut text_rect = item.rc_item;
        if pressed {
            text_rect.left += 1;
            text_rect.top += 1;
        }
        DrawTextW(
            item.hdc,
            text.as_ptr(),
            -1,
            &mut text_rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
        SelectObject(item.hdc, old_font);
    }
}

fn is_checkbox_id(id: i32) -> bool {
    matches!(
        id,
        ID_HTN
            | ID_DM
            | ID_FAST_HR
            | ID_SEVERE
            | ID_SKIP_REMINDER
            | ID_STARTUP_CHECK
            | ID_CLIPBOARD_AUTO
            | ID_SHORTCUT_SAVE_ORDER
            | ID_SETTINGS_ALT_F4
            | ID_SETTINGS_ALT_F4_NO
            | ID_SETTINGS_RES_1080
            | ID_SETTINGS_RES_2160
            | ID_SETTINGS_AUTO_REMOTE_SIGN
    )
}

fn is_resolution_option_id(id: i32) -> bool {
    id == ID_SETTINGS_RES_1080 || id == ID_SETTINGS_RES_2160
}

unsafe fn draw_owner_checkbox(item: &DrawItemStruct, checked: bool, focused: bool) {
    let dark = DARK_THEME.load(Ordering::SeqCst);
    let canvas_brush = CreateSolidBrush(theme_bg_color());
    FillRect(item.hdc, &item.rc_item, canvas_brush);
    DeleteObject(canvas_brush);

    let size = 16;
    let left = item.rc_item.left + 2;
    let top = item.rc_item.top + ((item.rc_item.bottom - item.rc_item.top - size) / 2);
    let border = if focused {
        rgb(0, 122, 255)
    } else if dark {
        rgb(99, 99, 102)
    } else {
        rgb(174, 174, 178)
    };
    let fill = if checked {
        if dark {
            rgb(10, 132, 255)
        } else {
            rgb(0, 122, 255)
        }
    } else if dark {
        rgb(44, 44, 46)
    } else {
        rgb(255, 255, 255)
    };

    let old_pen = SelectObject(item.hdc, GetStockObject(NULL_PEN));
    let border_brush = CreateSolidBrush(border);
    let old_brush = SelectObject(item.hdc, border_brush);
    RoundRect(item.hdc, left, top, left + size, top + size, 6, 6);
    let fill_brush = CreateSolidBrush(fill);
    SelectObject(item.hdc, fill_brush);
    RoundRect(
        item.hdc,
        left + 1,
        top + 1,
        left + size - 1,
        top + size - 1,
        5,
        5,
    );
    SelectObject(item.hdc, old_brush);
    SelectObject(item.hdc, old_pen);
    DeleteObject(fill_brush);
    DeleteObject(border_brush);

    if checked {
        let pen = CreatePen(PS_SOLID, 2, rgb(255, 255, 255));
        let previous_pen = SelectObject(item.hdc, pen);
        MoveToEx(item.hdc, left + 4, top + 8, null_mut());
        LineTo(item.hdc, left + 7, top + 11);
        LineTo(item.hdc, left + 13, top + 5);
        SelectObject(item.hdc, previous_pen);
        DeleteObject(pen);
    }

    SetBkMode(item.hdc, TRANSPARENT);
    SetTextColor(item.hdc, theme_text_color());
    let old_font = SelectObject(item.hdc, MAIN_FONT);
    let mut text_rect = item.rc_item;
    text_rect.left += 25;
    let text = wide(&get_window_text(item.hwnd_item));
    DrawTextW(
        item.hdc,
        text.as_ptr(),
        -1,
        &mut text_rect,
        DT_VCENTER | DT_SINGLELINE,
    );
    SelectObject(item.hdc, old_font);
}

unsafe fn draw_status_panel(item: &DrawItemStruct) {
    let dark = DARK_THEME.load(Ordering::SeqCst);
    let canvas_brush = CreateSolidBrush(theme_bg_color());
    FillRect(item.hdc, &item.rc_item, canvas_brush);
    DeleteObject(canvas_brush);

    let border = if dark {
        rgb(72, 72, 74)
    } else {
        rgb(209, 209, 214)
    };
    let fill = if dark {
        rgb(44, 44, 46)
    } else {
        rgb(255, 255, 255)
    };
    let old_pen = SelectObject(item.hdc, GetStockObject(NULL_PEN));
    let border_brush = CreateSolidBrush(border);
    let old_brush = SelectObject(item.hdc, border_brush);
    RoundRect(
        item.hdc,
        item.rc_item.left,
        item.rc_item.top,
        item.rc_item.right,
        item.rc_item.bottom,
        12,
        12,
    );
    let fill_brush = CreateSolidBrush(fill);
    SelectObject(item.hdc, fill_brush);
    RoundRect(
        item.hdc,
        item.rc_item.left + 1,
        item.rc_item.top + 1,
        item.rc_item.right - 1,
        item.rc_item.bottom - 1,
        10,
        10,
    );
    SelectObject(item.hdc, old_brush);
    SelectObject(item.hdc, old_pen);
    DeleteObject(fill_brush);
    DeleteObject(border_brush);

    SetBkMode(item.hdc, TRANSPARENT);
    SetTextColor(item.hdc, theme_secondary_text_color());
    let old_font = SelectObject(item.hdc, NOTE_FONT);
    let text = wide(&get_window_text(item.hwnd_item));
    let mut text_rect = item.rc_item;
    text_rect.left += 11;
    text_rect.right -= 10;
    text_rect.top += 7;
    text_rect.bottom -= 6;
    DrawTextW(item.hdc, text.as_ptr(), -1, &mut text_rect, DT_WORDBREAK);
    SelectObject(item.hdc, old_font);
}

unsafe fn layout_footer(hwnd: Hwnd) {
    let mut rect = Rect::default();
    if GetClientRect(hwnd, &mut rect) == 0 {
        return;
    }
    let height = (rect.bottom - rect.top).max(620);
    let main_x = 20;
    if APP.status != 0 {
        MoveWindow(APP.status, main_x, height - 88, 270, 50, 1);
    }
    if APP.version != 0 {
        MoveWindow(APP.version, main_x, height - 28, 164, 20, 1);
    }
    if APP.settings_button != 0 {
        MoveWindow(APP.settings_button, main_x + 238, height - 32, 28, 24, 1);
    }
    if APP.cleanup_button != 0 {
        MoveWindow(APP.cleanup_button, main_x + 184, height - 32, 48, 24, 1);
    }
}

#[repr(C)]
struct PaintStruct {
    hdc: Hdc,
    erase: i32,
    rc_paint: Rect,
    restore: i32,
    inc_update: i32,
    reserved: [u8; 32],
}

unsafe fn handle_command(id: i32) {
    match id {
        ID_SETTINGS_OPEN_MAIN => {
            show_settings_window();
            return;
        }
        ID_CLEANUP_RUNNING_APPS => {
            start_cleanup_running_apps();
            return;
        }
        ID_TAB_TEMPLATES => {
            show_template_page(true);
            set_status("请把光标放在书写病程的第一个字之前。");
            return;
        }
        ID_TAB_SHORTCUTS => {
            show_template_page(false);
            set_status("快捷操作已就绪。");
            return;
        }
        ID_TAB_API => {
            if load_settings().api_mode_enabled {
                show_api_page();
                set_status("API 模式已打开：可整理患者口语化描述。");
            } else {
                set_status("请先在托盘右键设置中打开 API 模式。");
            }
            return;
        }
        ID_API_SEND => {
            let patient = get_window_text(APP.api_patient);
            let description = get_window_text(APP.api_description);
            let prompt = get_window_text(APP.api_template_prompt);
            let _request_preview = format!(
                "patient={}; description={}; prompt={}",
                patient.trim(),
                description.trim(),
                prompt.trim()
            );
            API_DOC_GENERATED_MASK.store((1u32 << API_DOC_COUNT) - 1, Ordering::SeqCst);
            refresh_api_doc_rows();
            set_status("已模拟生成全部 API 模板；右键模板行可写回病历系统。");
            return;
        }
        ID_API_TYPE_RESULT => {
            let index = API_CONTEXT_DOC_INDEX.load(Ordering::SeqCst);
            if index < 0 {
                set_status("请先单击或右键选择一个模板。");
                return;
            }
            writeback_api_doc(index as usize);
            return;
        }
        ID_API_PATIENT_NEW => {
            SetWindowTextW(APP.api_patient, wide("新患者").as_ptr());
            SetWindowTextW(APP.api_description, wide("").as_ptr());
            reset_api_patient_workspace();
            SetFocus(APP.api_patient);
            set_status("已新建患者；请粘贴口语化病例描述。");
            return;
        }
        ID_API_PATIENT_ADD => {
            set_status("已保留当前患者信息；患者库持久化下一步接入。");
            return;
        }
        ID_API_PATIENT_DELETE => {
            SetWindowTextW(APP.api_patient, wide("").as_ptr());
            SetWindowTextW(APP.api_description, wide("").as_ptr());
            reset_api_patient_workspace();
            set_status("已清空当前患者。");
            return;
        }
        id if (ID_API_DOC_BASE..ID_API_DOC_BASE + API_DOC_COUNT as i32).contains(&id) => {
            let index = (id - ID_API_DOC_BASE) as usize;
            API_CONTEXT_DOC_INDEX.store(index as i32, Ordering::SeqCst);
            set_status(&format!("已选择 API 模板：{}", API_DOC_NAMES[index]));
            return;
        }
        ID_DISEASE_TOGGLE => {
            let show = !DISEASE_DROPDOWN_OPEN.load(Ordering::SeqCst);
            DISEASE_DROPDOWN_OPEN.store(show, Ordering::SeqCst);
            show_disease_dropdown(show);
            set_status(if show {
                "已展开疾病列表；单击一项后自动收起。"
            } else {
                "已收起疾病列表。"
            });
            return;
        }
        ID_THEME_TOGGLE => {
            let dark = !DARK_THEME.load(Ordering::SeqCst);
            set_theme(dark);
            save_settings(&read_settings_from_ui());
            set_status(if dark {
                "已切换为夜间灰色皮肤。"
            } else {
                "已切换为日间白色皮肤。"
            });
            return;
        }
        ID_DOCK_TOGGLE => {
            let enabled = !DOCK_ENABLED.load(Ordering::SeqCst);
            set_dock_enabled(enabled);
            save_settings(&read_settings_from_ui());
            set_status(if enabled {
                "已进入吸附模式：靠右或靠顶会自动隐藏，鼠标靠近边缘会弹出。"
            } else {
                "已关闭吸附模式：窗口可自由拖动，不再自动收回。"
            });
            return;
        }
        ID_STARTUP_CHECK => {
            let enabled = toggle_checkbox(ID_STARTUP_CHECK, APP.startup_check);
            let ok = set_startup_enabled(enabled);
            save_settings(&read_settings_from_ui());
            set_status(if ok {
                if enabled {
                    "已开启开机自启动。"
                } else {
                    "已关闭开机自启动。"
                }
            } else {
                "开机自启动设置失败。"
            });
            return;
        }
        ID_CLIPBOARD_AUTO => {
            let enabled = toggle_checkbox(ID_CLIPBOARD_AUTO, APP.clipboard_auto);
            update_feature_toggle_text(ID_CLIPBOARD_AUTO, enabled);
            save_settings(&read_settings_from_ui());
            set_status(if enabled {
                "剪贴板自动输入已开启；按 Ctrl+Alt+V 逐字输入。"
            } else {
                "剪贴板自动输入已关闭。"
            });
            return;
        }
        ID_SHORTCUT_MEDICAL_LAUNCH => {
            let settings = read_settings_from_ui();
            save_settings(&settings);
            queue_open_program_flow(
                settings.username,
                settings.password,
                OpenLaunchTarget::Medical,
            );
            return;
        }
        ID_SHORTCUT_ORDER_LAUNCH => {
            let settings = read_settings_from_ui();
            save_settings(&settings);
            queue_open_program_flow(
                settings.username,
                settings.password,
                OpenLaunchTarget::Order,
            );
            return;
        }
        ID_SHORTCUT_REMOTE_SIGN => {
            let settings = read_settings_from_ui();
            save_settings(&settings);
            start_remote_sign_flow(settings);
            return;
        }
        ID_SHORTCUT_VT => {
            start_clinical_path_flow();
            return;
        }
        ID_SHORTCUT_CLINICAL_CONTINUE => {
            start_clinical_path_continuous_flow();
            return;
        }
        ID_SHORTCUT_SAVE_ORDER => {
            let enabled = toggle_checkbox(ID_SHORTCUT_SAVE_ORDER, APP.save_order_hotkey);
            update_feature_toggle_text(ID_SHORTCUT_SAVE_ORDER, enabled);
            save_settings(&read_settings_from_ui());
            set_status(if enabled {
                "医嘱快捷保存已开启；按 Insert 执行。"
            } else {
                "医嘱快捷保存已关闭。"
            });
            return;
        }
        ID_ACCOUNT_QUERY => {
            show_account_query_dialog();
            return;
        }
        ID_ACCOUNT_DIALOG_CLOSE => {
            ShowWindow(APP.account_popup, 0);
            set_status("账号查询已关闭。");
            return;
        }
        ID_NURSING_LAUNCH => {
            let account = generated_nursing_account();
            save_settings(&read_settings_from_ui());
            start_nursing_flow(account);
            return;
        }
        ID_WARD_BUILDING => {
            let next = (selected_ward_building_index() + 1) % WARD_BUILDINGS.len();
            set_ward_selection(next, selected_ward_floor());
            let account = generated_nursing_account();
            save_settings(&read_settings_from_ui());
            set_status(&format!("远卓账号/密码：{}", account));
            return;
        }
        ID_WARD_FLOOR => {
            let floor = selected_ward_floor();
            let next = if floor >= 10 { 2 } else { floor + 1 };
            set_ward_selection(selected_ward_building_index(), next);
            let account = generated_nursing_account();
            save_settings(&read_settings_from_ui());
            set_status(&format!("远卓账号/密码：{}", account));
            return;
        }
        ID_ATTENDING_SUPERIOR => {
            toggle_superior_selector(0);
            return;
        }
        ID_CHIEF_SUPERIOR => {
            toggle_superior_selector(1);
            return;
        }
        ID_SHORTCUT_CREATE_ALL => {
            let base_time = course_base_datetime(APP.create_all_base_time);
            let settings = read_settings_from_ui();
            let attending = settings.attending_superior.clone();
            let chief = settings.chief_superior.clone();
            save_settings(&settings);
            start_create_courses_flow(CourseBatchKind::Preop, base_time, attending, chief);
            return;
        }
        ID_SHORTCUT_CREATE_POSTOP => {
            let surgery_time = course_base_datetime(APP.postop_base_time);
            let settings = read_settings_from_ui();
            let attending = settings.attending_superior.clone();
            let chief = settings.chief_superior.clone();
            save_settings(&settings);
            start_create_courses_flow(CourseBatchKind::Postop, surgery_time, attending, chief);
            return;
        }
        _ => {}
    }

    if let Some(button) = BUTTONS.iter().find(|t| t.id == id) {
        let options = read_options();
        let disease_index = selected_disease_index();
        let job = prepare_job(*button, options, disease_index);
        STOP_TYPING.store(false, Ordering::SeqCst);
        let start_delay_secs;
        if reminder_enabled() {
            set_status("请把光标放在书写病程的第一个字之前。");
            MessageBoxW(
                APP.hwnd,
                wide("点击“确定”后，请在 3 秒内把光标放在书写病程的第一个字之前。程序会逐字模拟键盘输入。输入过程中按右Ctrl+右Alt可停止。").as_ptr(),
                wide("准备模拟输入").as_ptr(),
                MB_OK,
            );
            start_delay_secs = 2;
        } else {
            set_status("3 秒后开始；请把光标放在书写病程的第一个字之前。");
            start_delay_secs = 3;
        }
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(start_delay_secs));
            run_job(job);
        });
        return;
    }

    match id {
        ID_STOP => {
            STOP_TYPING.store(true, Ordering::SeqCst);
            set_status("已请求停止输入");
        }
        ID_SKIP_REMINDER => {
            toggle_checkbox(ID_SKIP_REMINDER, APP.skip_reminder);
            save_settings(&read_settings_from_ui());
            set_status("设置已保存。");
        }
        ID_HTN | ID_DM | ID_FAST_HR | ID_SEVERE => {
            let control = match id {
                ID_HTN => APP.htn,
                ID_DM => APP.dm,
                ID_FAST_HR => APP.fast_hr,
                _ => APP.severe,
            };
            toggle_checkbox(id, control);
            set_status("选项已更新。点击模板按钮开始输入。");
        }
        _ => {}
    }
}

fn checkbox_bit(id: i32) -> u32 {
    match id {
        ID_HTN => 1 << 0,
        ID_DM => 1 << 1,
        ID_FAST_HR => 1 << 2,
        ID_SEVERE => 1 << 3,
        ID_SKIP_REMINDER => 1 << 4,
        ID_STARTUP_CHECK => 1 << 5,
        ID_CLIPBOARD_AUTO => 1 << 6,
        ID_SHORTCUT_SAVE_ORDER => 1 << 7,
        ID_SETTINGS_ALT_F4 => 1 << 8,
        ID_SETTINGS_ALT_F4_NO => 1 << 9,
        ID_SETTINGS_AUTO_REMOTE_SIGN => 1 << 10,
        ID_SETTINGS_RES_1080 => 1 << 11,
        ID_SETTINGS_RES_2160 => 1 << 12,
        _ => 0,
    }
}

fn checkbox_checked(id: i32) -> bool {
    let bit = checkbox_bit(id);
    bit != 0 && CHECKBOX_STATES.load(Ordering::SeqCst) & bit != 0
}

unsafe fn set_checkbox(id: i32, control: Hwnd, checked: bool) {
    let bit = checkbox_bit(id);
    if checked {
        CHECKBOX_STATES.fetch_or(bit, Ordering::SeqCst);
    } else {
        CHECKBOX_STATES.fetch_and(!bit, Ordering::SeqCst);
    }
    InvalidateRect(control, null(), 1);
}

unsafe fn toggle_checkbox(id: i32, control: Hwnd) -> bool {
    let checked = checkbox_checked(id);
    let next = !checked;
    set_checkbox(id, control, next);
    next
}

unsafe fn update_feature_toggle_text(id: i32, enabled: bool) {
    let (control, title, shortcut) = if id == ID_SHORTCUT_SAVE_ORDER {
        (APP.save_order_hotkey, "医嘱快捷保存", "Insert")
    } else {
        (APP.clipboard_auto, "剪贴板自动输入", "Ctrl+Alt+V")
    };
    if control != 0 {
        SetWindowTextW(
            control,
            wide(&format!(
                "{} · {}\n{}",
                title,
                if enabled { "已开启" } else { "已关闭" },
                shortcut
            ))
            .as_ptr(),
        );
        InvalidateRect(control, null(), 1);
    }
}

unsafe fn show_account_query_dialog() {
    if APP.account_popup == 0 {
        return;
    }
    let width = 350;
    let height = 456;
    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let screen_h = GetSystemMetrics(SM_CYSCREEN);
    SetWindowPos(
        APP.account_popup,
        HWND_TOP,
        ((screen_w - width) / 2).max(0),
        ((screen_h - height) / 2).max(0),
        width,
        height,
        0,
    );
    ShowWindow(APP.account_popup, 5);
    BringWindowToTop(APP.account_popup);
    SetForegroundWindow(APP.account_popup);
    SetFocus(APP.account_popup);
    set_status("已打开账号查询。");
}

unsafe fn reminder_enabled() -> bool {
    !checkbox_checked(ID_SKIP_REMINDER)
}

unsafe fn load_settings_into_ui() {
    let settings = load_settings();
    SetWindowTextW(APP.login_user, wide(&settings.username).as_ptr());
    SetWindowTextW(APP.login_pass, wide(&settings.password).as_ptr());
    SetWindowTextW(
        APP.clinical_loop_count,
        wide(&settings.clinical_loop_count).as_ptr(),
    );
    let base_time = if settings.create_all_base_time.trim().is_empty() {
        default_create_all_base_time()
    } else {
        settings.create_all_base_time
    };
    set_date_picker_value(APP.create_all_base_time, &base_time);
    let postop_time = if settings.postop_base_time.trim().is_empty() {
        default_create_all_base_time()
    } else {
        settings.postop_base_time
    };
    set_date_picker_value(APP.postop_base_time, &postop_time);
    let course_config = load_create_courses_config();
    let attending = if settings.attending_superior.trim().is_empty() {
        course_config.attending_superior
    } else {
        settings.attending_superior
    };
    let chief = if settings.chief_superior.trim().is_empty() {
        course_config.chief_superior
    } else {
        settings.chief_superior
    };
    set_superior_button_text(0, &attending, false);
    set_superior_button_text(1, &chief, false);
    set_checkbox(ID_SKIP_REMINDER, APP.skip_reminder, settings.skip_reminder);
    let startup_enabled = is_startup_enabled();
    set_checkbox(ID_STARTUP_CHECK, APP.startup_check, startup_enabled);
    set_checkbox(
        ID_CLIPBOARD_AUTO,
        APP.clipboard_auto,
        settings.clipboard_auto,
    );
    set_checkbox(
        ID_SHORTCUT_SAVE_ORDER,
        APP.save_order_hotkey,
        settings.save_order_hotkey,
    );
    update_feature_toggle_text(ID_CLIPBOARD_AUTO, settings.clipboard_auto);
    update_feature_toggle_text(ID_SHORTCUT_SAVE_ORDER, settings.save_order_hotkey);
    set_selected_disease(settings.disease_index);
    set_ward_selection(settings.ward_building_index, settings.ward_floor);
    set_theme(settings.dark_theme);
    update_api_mode_visibility(settings.api_mode_enabled);
    DOCK_ENABLED.store(settings.dock_enabled, Ordering::SeqCst);
    DOCK_HIDE_DELAY_MS.store(settings.dock_hide_delay_ms, Ordering::SeqCst);
    LAUNCH_RESOLUTION_PROFILE.store(settings.launch_resolution_profile.min(1), Ordering::SeqCst);
    CLINICAL_PATH_RESOLUTION_PROFILE.store(
        settings.clinical_path_resolution_profile.min(1),
        Ordering::SeqCst,
    );
    TASKBAR_VISIBLE.store(settings.taskbar_visible, Ordering::SeqCst);
    DOCK_EDGE.store(DOCK_EDGE_RIGHT, Ordering::SeqCst);
    DOCK_VISIBLE.store(true, Ordering::SeqCst);
    DOCK_TARGET_VISIBLE.store(true, Ordering::SeqCst);
    update_dock_button();
}

unsafe fn update_api_mode_visibility(enabled: bool) {
    let _ = enabled;
    show_template_page(true);
}

unsafe fn read_settings_from_ui() -> Settings {
    let (window_x, window_y, window_w, window_h) = current_window_bounds();
    let existing = load_settings();
    Settings {
        username: get_window_text(APP.login_user),
        password: get_window_text(APP.login_pass),
        skip_reminder: checkbox_checked(ID_SKIP_REMINDER),
        window_x,
        window_y,
        window_w,
        window_h,
        disease_index: selected_disease_index(),
        dark_theme: DARK_THEME.load(Ordering::SeqCst),
        dock_enabled: DOCK_ENABLED.load(Ordering::SeqCst),
        taskbar_visible: TASKBAR_VISIBLE.load(Ordering::SeqCst),
        startup_enabled: checkbox_checked(ID_STARTUP_CHECK),
        clipboard_auto: checkbox_checked(ID_CLIPBOARD_AUTO),
        save_order_hotkey: checkbox_checked(ID_SHORTCUT_SAVE_ORDER),
        clinical_loop_count: normalized_loop_count_text(&get_window_text(APP.clinical_loop_count)),
        create_all_base_time: get_date_picker_value(APP.create_all_base_time),
        postop_base_time: get_date_picker_value(APP.postop_base_time),
        attending_superior: selected_superior_name(0),
        chief_superior: selected_superior_name(1),
        ward_building_index: selected_ward_building_index(),
        ward_floor: selected_ward_floor(),
        medical_system_path: existing.medical_system_path,
        order_system_path: existing.order_system_path,
        nursing_system_path: existing.nursing_system_path,
        remote_sign_server: existing.remote_sign_server,
        remote_sign_port: existing.remote_sign_port,
        auto_remote_sign: existing.auto_remote_sign,
        dock_hide_delay_ms: existing.dock_hide_delay_ms,
        launch_resolution_profile: existing.launch_resolution_profile.min(1),
        clinical_path_resolution_profile: existing.clinical_path_resolution_profile.min(1),
        launch_alt_f4: existing.launch_alt_f4,
        api_mode_enabled: existing.api_mode_enabled,
        deepseek_api_key: existing.deepseek_api_key,
        api_prompt_text: existing.api_prompt_text,
    }
}

unsafe fn current_window_bounds() -> (i32, i32, i32, i32) {
    let mut rect = Rect::default();
    if APP.hwnd != 0 && GetWindowRect(APP.hwnd, &mut rect) != 0 {
        (
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
        )
    } else {
        (120, 80, 326, 680)
    }
}

unsafe fn set_date_picker_value(control: Hwnd, value: &str) {
    let parts = parse_datetime_parts(value)
        .map(|parts| [parts[0], parts[1], parts[2]])
        .or_else(|| parse_date_parts(value));
    let Some([year, month, day]) = parts else {
        return;
    };
    let mut system_time = WinSystemTime {
        year: year as u16,
        month: month as u16,
        day: day as u16,
        ..Default::default()
    };
    SendMessageW(
        control,
        DTM_SETSYSTEMTIME,
        GDT_VALID,
        &mut system_time as *mut WinSystemTime as Lparam,
    );
}

unsafe fn get_date_picker_value(control: Hwnd) -> String {
    let mut system_time = WinSystemTime::default();
    let result = SendMessageW(
        control,
        DTM_GETSYSTEMTIME,
        0,
        &mut system_time as *mut WinSystemTime as Lparam,
    );
    if result == GDT_VALID as Lresult {
        format!(
            "{:04}-{:02}-{:02}",
            system_time.year, system_time.month, system_time.day
        )
    } else {
        default_create_all_base_time()
            .split_whitespace()
            .next()
            .unwrap_or("2026-01-01")
            .to_string()
    }
}

unsafe fn course_base_datetime(control: Hwnd) -> String {
    format!("{} 20:00:00", get_date_picker_value(control))
}

unsafe fn get_window_text(hwnd: Hwnd) -> String {
    let len = SendMessageW(hwnd, WM_GETTEXTLENGTH, 0, 0) as usize;
    let mut buf = vec![0u16; len + 1];
    SendMessageW(hwnd, WM_GETTEXT, buf.len(), buf.as_mut_ptr() as Lparam);
    String::from_utf16_lossy(&buf[..len])
}

fn load_settings() -> Settings {
    let mut settings = Settings::default();
    if let Ok(text) = fs::read_to_string(settings_path()) {
        for line in text.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key {
                    "username" => settings.username = decode_setting(value),
                    "password" => settings.password = decode_setting(value),
                    "skip_reminder" => settings.skip_reminder = value.trim() == "1",
                    "window_x" => settings.window_x = value.trim().parse().unwrap_or(0),
                    "window_y" => settings.window_y = value.trim().parse().unwrap_or(0),
                    "window_w" => settings.window_w = value.trim().parse().unwrap_or(0),
                    "window_h" => settings.window_h = value.trim().parse().unwrap_or(0),
                    "disease_index" => settings.disease_index = value.trim().parse().unwrap_or(0),
                    "dark_theme" => settings.dark_theme = value.trim() == "1",
                    "dock_enabled" => settings.dock_enabled = value.trim() != "0",
                    "taskbar_visible" => settings.taskbar_visible = value.trim() != "0",
                    "startup_enabled" => settings.startup_enabled = value.trim() == "1",
                    "clipboard_auto" => settings.clipboard_auto = value.trim() == "1",
                    "save_order_hotkey" => settings.save_order_hotkey = value.trim() == "1",
                    "clinical_loop_count" => {
                        settings.clinical_loop_count =
                            normalized_loop_count_text(&decode_setting(value))
                    }
                    "create_all_base_time" => settings.create_all_base_time = decode_setting(value),
                    "postop_base_time" => settings.postop_base_time = decode_setting(value),
                    "attending_superior" => settings.attending_superior = decode_setting(value),
                    "chief_superior" => settings.chief_superior = decode_setting(value),
                    "ward_building_index" => {
                        settings.ward_building_index = value.trim().parse().unwrap_or(2)
                    }
                    "ward_floor" => settings.ward_floor = value.trim().parse().unwrap_or(5),
                    "medical_system_path" => settings.medical_system_path = decode_setting(value),
                    "order_system_path" => settings.order_system_path = decode_setting(value),
                    "nursing_system_path" => settings.nursing_system_path = decode_setting(value),
                    "remote_sign_server" => settings.remote_sign_server = decode_setting(value),
                    "remote_sign_port" => {
                        settings.remote_sign_port = normalized_remote_sign_port(value)
                    }
                    "auto_remote_sign" => settings.auto_remote_sign = value.trim() == "1",
                    "dock_hide_delay_ms" => {
                        settings.dock_hide_delay_ms = normalized_dock_hide_delay(value)
                    }
                    "launch_resolution_profile" => {
                        settings.launch_resolution_profile =
                            value.trim().parse::<usize>().unwrap_or(0).min(1)
                    }
                    "clinical_path_resolution_profile" => {
                        settings.clinical_path_resolution_profile =
                            value.trim().parse::<usize>().unwrap_or(0).min(1)
                    }
                    "launch_alt_f4" => settings.launch_alt_f4 = value.trim() == "1",
                    "api_mode_enabled" => settings.api_mode_enabled = value.trim() == "1",
                    "deepseek_api_key" => settings.deepseek_api_key = decode_setting(value),
                    "api_prompt_text" => settings.api_prompt_text = decode_setting(value),
                    _ => {}
                }
            }
        }
    }
    if !text_contains_key(
        "dock_enabled",
        &fs::read_to_string(settings_path()).unwrap_or_default(),
    ) {
        settings.dock_enabled = true;
    }
    if !text_contains_key(
        "taskbar_visible",
        &fs::read_to_string(settings_path()).unwrap_or_default(),
    ) {
        settings.taskbar_visible = true;
    }
    settings
}

fn save_settings(settings: &Settings) {
    let text = format!(
        "username={}\npassword={}\nskip_reminder={}\nwindow_x={}\nwindow_y={}\nwindow_w={}\nwindow_h={}\ndisease_index={}\ndark_theme={}\ndock_enabled={}\ntaskbar_visible={}\nstartup_enabled={}\nclipboard_auto={}\nsave_order_hotkey={}\nclinical_loop_count={}\ncreate_all_base_time={}\npostop_base_time={}\nattending_superior={}\nchief_superior={}\nward_building_index={}\nward_floor={}\nmedical_system_path={}\norder_system_path={}\nnursing_system_path={}\nremote_sign_server={}\nremote_sign_port={}\nauto_remote_sign={}\ndock_hide_delay_ms={}\nlaunch_resolution_profile={}\nclinical_path_resolution_profile={}\nlaunch_alt_f4={}\napi_mode_enabled={}\ndeepseek_api_key={}\napi_prompt_text={}\n",
        encode_setting(&settings.username),
        encode_setting(&settings.password),
        if settings.skip_reminder { "1" } else { "0" },
        settings.window_x,
        settings.window_y,
        settings.window_w,
        settings.window_h,
        settings.disease_index.min(DISEASE_ITEMS.len() - 1),
        if settings.dark_theme { "1" } else { "0" },
        if settings.dock_enabled { "1" } else { "0" },
        if settings.taskbar_visible { "1" } else { "0" },
        if settings.startup_enabled { "1" } else { "0" },
        if settings.clipboard_auto { "1" } else { "0" },
        if settings.save_order_hotkey { "1" } else { "0" },
        encode_setting(&settings.clinical_loop_count),
        encode_setting(&settings.create_all_base_time),
        encode_setting(&settings.postop_base_time),
        encode_setting(&settings.attending_superior),
        encode_setting(&settings.chief_superior),
        settings.ward_building_index.min(WARD_BUILDINGS.len() - 1),
        settings.ward_floor.clamp(2, 10),
        encode_setting(&settings.medical_system_path),
        encode_setting(&settings.order_system_path),
        encode_setting(&settings.nursing_system_path),
        encode_setting(&settings.remote_sign_server),
        normalized_remote_sign_port(&settings.remote_sign_port.to_string()),
        if settings.auto_remote_sign { "1" } else { "0" },
        settings.dock_hide_delay_ms.min(5_000),
        settings.launch_resolution_profile.min(1),
        settings.clinical_path_resolution_profile.min(1),
        if settings.launch_alt_f4 { "1" } else { "0" },
        if settings.api_mode_enabled { "1" } else { "0" },
        encode_setting(&settings.deepseek_api_key),
        encode_setting(&settings.api_prompt_text)
    );
    let _ = fs::write(settings_path(), text);
}

fn text_contains_key(key: &str, text: &str) -> bool {
    text.lines().any(|line| {
        line.split_once('=')
            .map(|(left, _)| left == key)
            .unwrap_or(false)
    })
}

fn startup_registry_path() -> &'static str {
    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run"
}

fn startup_exe_path() -> Option<PathBuf> {
    std::env::current_exe().ok()
}

fn is_startup_enabled() -> bool {
    Command::new("reg")
        .args(["query", startup_registry_path(), "/v", STARTUP_VALUE_NAME])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn set_startup_enabled(enabled: bool) -> bool {
    if enabled {
        let Some(path) = startup_exe_path() else {
            return false;
        };
        let value = format!("\"{}\"", path.display());
        Command::new("reg")
            .arg("add")
            .arg(startup_registry_path())
            .arg("/v")
            .arg(STARTUP_VALUE_NAME)
            .arg("/t")
            .arg("REG_SZ")
            .arg("/d")
            .arg(value)
            .arg("/f")
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    } else {
        Command::new("reg")
            .args([
                "delete",
                startup_registry_path(),
                "/v",
                STARTUP_VALUE_NAME,
                "/f",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map(|status| status.success())
            .unwrap_or(true)
    }
}

fn normalized_loop_count_text(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return String::new();
    }
    match value.parse::<u32>() {
        Ok(count) if (2..=10).contains(&count) => count.to_string(),
        _ => String::new(),
    }
}

unsafe fn clinical_loop_count_limit() -> Option<u32> {
    normalized_loop_count_text(&get_window_text(APP.clinical_loop_count))
        .parse::<u32>()
        .ok()
}

fn encode_setting(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('\n', "%0A")
        .replace('\r', "%0D")
        .replace('=', "%3D")
}

fn decode_setting(value: &str) -> String {
    value
        .replace("%3D", "=")
        .replace("%0D", "\r")
        .replace("%0A", "\n")
        .replace("%25", "%")
}

fn settings_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.join("settings.txt");
        }
    }
    PathBuf::from("settings.txt")
}

fn error_log_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("error.log")))
        .unwrap_or_else(|| PathBuf::from("error.log"))
}

fn log_timestamp() -> String {
    let local_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64 + 8 * 3600)
        .unwrap_or(0);
    let date = date_from_unix_days(local_seconds.div_euclid(86_400));
    let seconds = local_seconds.rem_euclid(86_400);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        date.year,
        date.month,
        date.day,
        seconds / 3600,
        (seconds % 3600) / 60,
        seconds % 60
    )
}

fn log_event(message: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(error_log_path())
    {
        let _ = writeln!(file, "[{}] {}", log_timestamp(), message);
    }
}

fn create_courses_config_path() -> PathBuf {
    editable_text_path("create_courses_config.txt")
}

fn default_create_all_base_time() -> String {
    let local_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64 + 8 * 3600)
        .unwrap_or(0);
    let date = date_from_unix_days(local_seconds.div_euclid(86_400));
    format!(
        "{:04}-{:02}-{:02} 20:00:00",
        date.year, date.month, date.day
    )
}

fn parse_base_date(value: &str) -> Result<SimpleDate, String> {
    let date_text = value
        .trim()
        .split_whitespace()
        .next()
        .ok_or_else(|| "请输入YYYY-MM-DD HH:MM:SS".to_string())?;
    let parts: Vec<&str> = date_text.split('-').collect();
    if parts.len() != 3 {
        return Err("请输入YYYY-MM-DD HH:MM:SS".to_string());
    }
    let date = SimpleDate {
        year: parts[0].parse().map_err(|_| "年份无效".to_string())?,
        month: parts[1].parse().map_err(|_| "月份无效".to_string())?,
        day: parts[2].parse().map_err(|_| "日期无效".to_string())?,
    };
    if date.year < 2000
        || date.year > 2100
        || date.month == 0
        || date.month > 12
        || date.day == 0
        || date.day > days_in_month(date.year, date.month)
    {
        return Err("日期超出有效范围".to_string());
    }
    Ok(date)
}

fn parse_base_datetime(value: &str) -> Result<(SimpleDate, String), String> {
    let date = parse_base_date(value)?;
    let parts =
        parse_datetime_parts(value).ok_or_else(|| "请输入完整的YYYY-MM-DD HH:MM:SS".to_string())?;
    Ok((
        date,
        format!("{:02}:{:02}:{:02}", parts[3], parts[4], parts[5]),
    ))
}

fn add_days(mut date: SimpleDate, days: u32) -> SimpleDate {
    for _ in 0..days {
        date.day += 1;
        if date.day > days_in_month(date.year, date.month) {
            date.day = 1;
            date.month += 1;
            if date.month > 12 {
                date.month = 1;
                date.year += 1;
            }
        }
    }
    date
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) => 29,
        2 => 28,
        _ => 0,
    }
}

fn date_from_unix_days(days: i64) -> SimpleDate {
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    }
    .div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }
    SimpleDate {
        year: year as i32,
        month: month as u32,
        day: day as u32,
    }
}

fn disease_placeholder_path() -> PathBuf {
    editable_text_path("disease_placeholders.txt")
}

fn editable_text_path(relative: impl AsRef<Path>) -> PathBuf {
    editable_text_root().join(relative.as_ref())
}

fn editable_text_root() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let beside_exe = dir.join(EDITABLE_TEXT_FOLDER);
            let project_root = dir.join("..").join("..").join(EDITABLE_TEXT_FOLDER);
            if project_root.exists() {
                return project_root;
            }
            if beside_exe.exists() {
                return beside_exe;
            }
            return beside_exe;
        }
    }
    PathBuf::from(EDITABLE_TEXT_FOLDER)
}

fn ensure_editable_text_files() {
    let files = [
        (
            "create_courses_config.txt",
            include_str!("../可修改文本/create_courses_config.txt"),
        ),
        (
            "disease_placeholders.txt",
            include_str!("../可修改文本/disease_placeholders.txt"),
        ),
        ("短语集.txt", include_str!("../可修改文本/短语集.txt")),
        ("固定文本.txt", include_str!("../可修改文本/固定文本.txt")),
        (
            "病程模板/director_round.txt",
            include_str!("../可修改文本/病程模板/director_round.txt"),
        ),
        (
            "病程模板/discharge_certificate.txt",
            include_str!("../可修改文本/病程模板/discharge_certificate.txt"),
        ),
        (
            "病程模板/discharge_record.txt",
            include_str!("../可修改文本/病程模板/discharge_record.txt"),
        ),
        (
            "病程模板/first_visit.txt",
            include_str!("../可修改文本/病程模板/first_visit.txt"),
        ),
        (
            "病程模板/post_op_daily.txt",
            include_str!("../可修改文本/病程模板/post_op_daily.txt"),
        ),
        (
            "病程模板/post_op_first.txt",
            include_str!("../可修改文本/病程模板/post_op_first.txt"),
        ),
        (
            "病程模板/preop_summary.txt",
            include_str!("../可修改文本/病程模板/preop_summary.txt"),
        ),
        (
            "病程模板/senior_round.txt",
            include_str!("../可修改文本/病程模板/senior_round.txt"),
        ),
    ];
    for (relative, content) in files {
        let path = editable_text_path(relative);
        if path.exists() {
            continue;
        }
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, content);
    }
    ensure_disease_template_files();
}

fn ensure_disease_template_files() {
    let phrases = load_phrases();
    let fixed_texts = load_fixed_texts();
    for (disease_index, disease) in DISEASE_ITEMS.iter().enumerate() {
        let legacy = load_legacy_disease_placeholders(disease_index);
        for source in DISEASE_TEMPLATE_SOURCES {
            let path = disease_template_path(disease, source);
            if path.exists() {
                continue;
            }
            let base = fs::read_to_string(template_path(source)).unwrap_or_default();
            let mut text = String::from("# 可修改病程内容\n# 方括号内为字段名称；直接修改字段下方文字即可。\n\n[模板正文]\n");
            text.push_str(&base);
            text.push_str("\n\n[鉴别诊断]\n");
            text.push_str(&legacy.differential);
            text.push_str("\n\n[诊疗计划]\n");
            text.push_str(&legacy.treatment_plan);
            text.push_str("\n\n[上级医师指示]\n");
            text.push_str(&legacy.senior_instruction);
            text.push_str("\n\n[查体]\n");
            text.push_str(&legacy.exam);

            for key in template_field_keys(source) {
                let value = fixed_texts
                    .get(*key)
                    .or_else(|| phrases.get(*key))
                    .map(String::as_str)
                    .unwrap_or_default();
                text.push_str("\n\n[");
                text.push_str(key);
                text.push_str("]\n");
                text.push_str(value);
            }

            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(path, text);
        }
    }
}

fn template_field_keys(source: &str) -> &'static [&'static str] {
    match source {
        "first_visit.txt" => &[
            "first_course_type",
            "first_course_differential_fallback",
            "first_course_plan_fallback",
        ],
        "senior_round.txt" => &[
            "bq2", "zs2", "jws1", "ct2", "jc2", "td2", "zd2", "yb2", "jh2",
        ],
        "post_op_daily.txt" => &["sh4", "post_daily_status", "ct3", "jc3", "sh5"],
        "post_senior_first.txt" => &[
            "post_senior_summary",
            "post_senior_complaint",
            "post_senior_exam",
            "post_senior_tests",
            "post_senior_plan",
        ],
        "preop_discussion.txt" => &[
            "preop_anesthesia",
            "preop_history",
            "preop_attending_speech",
            "preop_chief_speech",
            "preop_conclusion",
        ],
        "discharge_certificate.txt" => &["discharge_advice_default"],
        _ => &[],
    }
}

unsafe fn read_options() -> Options {
    Options {
        hypertension: checkbox_checked(ID_HTN),
        diabetes: checkbox_checked(ID_DM),
        fast_hr: checkbox_checked(ID_FAST_HR),
        severe: checkbox_checked(ID_SEVERE),
    }
}

enum InputJob {
    Text(String),
    Flow {
        kind: FlowKind,
        options: Options,
        disease_index: usize,
    },
}

fn prepare_job(button: ButtonDef, options: Options, disease_index: usize) -> InputJob {
    match button.action {
        ButtonAction::Template(file) => {
            let template = load_disease_template(disease_index, file);
            InputJob::Text(render_template(&template, options))
        }
        ButtonAction::Flow(kind) => InputJob::Flow {
            kind,
            options,
            disease_index,
        },
    }
}

fn run_job(job: InputJob) {
    match job {
        InputJob::Text(text) => type_text(&text),
        InputJob::Flow {
            kind,
            options,
            disease_index,
        } => run_builtin_flow(kind, options, disease_index),
    }
}

fn load_template(file: &str) -> String {
    fs::read_to_string(template_path(file)).unwrap_or_else(|_| format!("未找到模板文件：{}", file))
}

fn template_path(file: &str) -> PathBuf {
    editable_text_path(PathBuf::from(TEMPLATE_TEXT_FOLDER).join(file))
}

fn disease_template_file_name(file: &str) -> &'static str {
    match file {
        "first_visit.txt" => "首次病程.txt",
        "senior_round.txt" => "首次查房.txt",
        "director_round.txt" => "主任查房.txt",
        "preop_summary.txt" => "术前小结.txt",
        "preop_discussion.txt" => "术前讨论.txt",
        "post_op_first.txt" => "术后首程.txt",
        "post_senior_first.txt" => "术后首次上级查房.txt",
        "post_op_daily.txt" => "术后日常.txt",
        "discharge_certificate.txt" => "诊断证明.txt",
        "discharge_record.txt" => "出院记录.txt",
        _ => "未分类模板.txt",
    }
}

const DISEASE_TEMPLATE_SOURCES: [&str; 10] = [
    "first_visit.txt",
    "senior_round.txt",
    "director_round.txt",
    "preop_summary.txt",
    "preop_discussion.txt",
    "post_op_first.txt",
    "post_senior_first.txt",
    "post_op_daily.txt",
    "discharge_certificate.txt",
    "discharge_record.txt",
];

fn disease_template_path(disease: &str, file: &str) -> PathBuf {
    editable_text_path(
        PathBuf::from(DISEASE_TEMPLATE_FOLDER)
            .join(disease)
            .join(disease_template_file_name(file)),
    )
}

fn parse_bracketed_fields(text: &str) -> HashMap<String, String> {
    let mut fields = HashMap::new();
    let mut current: Option<String> = None;
    let mut value = String::new();
    for raw in text.lines() {
        let line = raw.trim_end_matches('\r');
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed.len() > 2 {
            if let Some(key) = current.take() {
                fields.insert(key, value.trim().to_string());
                value.clear();
            }
            current = Some(trimmed[1..trimmed.len() - 1].trim().to_string());
        } else if current.is_some() {
            value.push_str(line);
            value.push('\n');
        }
    }
    if let Some(key) = current {
        fields.insert(key, value.trim().to_string());
    }
    fields
}

fn load_disease_template_fields(disease_index: usize, file: &str) -> HashMap<String, String> {
    let disease = DISEASE_ITEMS[disease_index.min(DISEASE_ITEMS.len() - 1)];
    fs::read_to_string(disease_template_path(disease, file))
        .map(|text| parse_bracketed_fields(&text))
        .unwrap_or_default()
}

fn load_disease_template(disease_index: usize, file: &str) -> String {
    let fields = load_disease_template_fields(disease_index, file);
    fields
        .get("模板正文")
        .cloned()
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| load_template(file))
}

fn field_or_fallback<'a>(
    fields: &'a HashMap<String, String>,
    key: &str,
    fallback: Option<&'a String>,
) -> Option<&'a str> {
    fields
        .get(key)
        .map(String::as_str)
        .filter(|text| !text.trim().is_empty())
        .or_else(|| {
            fallback
                .map(String::as_str)
                .filter(|text| !text.trim().is_empty())
        })
}

fn script_path(file: &str) -> PathBuf {
    asset_path("scripts", file)
}

fn asset_path(folder: &str, file: &str) -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let beside_exe = dir.join(folder).join(file);
            if beside_exe.exists() {
                return beside_exe;
            }
            let project_root = dir.join("..").join("..").join(folder).join(file);
            if project_root.exists() {
                return project_root;
            }
        }
    }
    PathBuf::from(folder).join(file)
}

fn load_phrases() -> HashMap<String, String> {
    let mut phrases = HashMap::new();
    let Ok(text) = fs::read_to_string(editable_text_path("短语集.txt")) else {
        return phrases;
    };

    let mut current_key: Option<String> = None;
    let mut current_value = String::new();

    for raw in text.lines() {
        let line = raw.trim_end_matches('\r');
        if line.trim_start().starts_with(';') || line.trim().is_empty() {
            continue;
        }

        if let Some((left, right)) = line.split_once('=') {
            if let Some(key) = current_key.take() {
                phrases.insert(key, current_value.trim_end().to_string());
                current_value.clear();
            }

            let Some((code, order)) = left.split_once(',') else {
                continue;
            };
            let key = format!("{}{}", code.trim(), order.trim()).to_lowercase();
            if right.is_empty() {
                current_key = Some(key);
            } else {
                phrases.insert(key, right.to_string());
            }
        } else if current_key.is_some() {
            current_value.push_str(line);
            current_value.push('\n');
        }
    }

    if let Some(key) = current_key {
        phrases.insert(key, current_value.trim_end().to_string());
    }

    phrases
}

fn load_fixed_texts() -> HashMap<String, String> {
    let Ok(text) = fs::read_to_string(editable_text_path("固定文本.txt")) else {
        return HashMap::new();
    };
    text.lines()
        .filter_map(|raw| {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            Some((key.trim().to_string(), value.trim().replace("\\n", "\n")))
        })
        .collect()
}

#[derive(Default)]
struct DiseasePlaceholderSet {
    senior_instruction: String,
    exam: String,
    treatment_plan: String,
    differential: String,
}

#[derive(Clone, Copy)]
enum PlaceholderField {
    TreatmentPlan,
    Differential,
}

fn load_legacy_disease_placeholders(disease_index: usize) -> DiseasePlaceholderSet {
    let disease = DISEASE_ITEMS[disease_index.min(DISEASE_ITEMS.len() - 1)];
    let Ok(text) = fs::read_to_string(disease_placeholder_path()) else {
        return DiseasePlaceholderSet::default();
    };

    let mut in_section = false;
    let mut result = DiseasePlaceholderSet::default();
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_section = &line[1..line.len() - 1] == disease;
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "上级医师指示" => result.senior_instruction = value.trim().to_string(),
                "查体" => result.exam = value.trim().to_string(),
                "诊疗计划" => result.treatment_plan = value.trim().to_string(),
                "鉴别诊断" => result.differential = value.trim().to_string(),
                _ => {}
            }
        }
    }
    result
}

fn load_disease_placeholders(disease_index: usize, template_file: &str) -> DiseasePlaceholderSet {
    let legacy = load_legacy_disease_placeholders(disease_index);
    let fields = load_disease_template_fields(disease_index, template_file);
    DiseasePlaceholderSet {
        senior_instruction: field_or_fallback(
            &fields,
            "上级医师指示",
            Some(&legacy.senior_instruction),
        )
        .unwrap_or_default()
        .to_string(),
        exam: field_or_fallback(&fields, "查体", Some(&legacy.exam))
            .unwrap_or_default()
            .to_string(),
        treatment_plan: field_or_fallback(&fields, "诊疗计划", Some(&legacy.treatment_plan))
            .unwrap_or_default()
            .to_string(),
        differential: field_or_fallback(&fields, "鉴别诊断", Some(&legacy.differential))
            .unwrap_or_default()
            .to_string(),
    }
}

fn placeholder_text<'a>(
    placeholders: &'a DiseasePlaceholderSet,
    field: PlaceholderField,
) -> &'a str {
    match field {
        PlaceholderField::TreatmentPlan => &placeholders.treatment_plan,
        PlaceholderField::Differential => &placeholders.differential,
    }
}

#[derive(Clone, Copy)]
enum FlowStep {
    Delay(u64),
    Key(&'static str),
    RepeatKey(&'static str, usize),
    Editable(&'static str),
    Phrase(&'static str),
    Placeholder(PlaceholderField, &'static str),
    Vital(VitalField),
}

#[derive(Clone, Copy)]
enum VitalField {
    Sbp,
    Dbp,
    Hr,
    Rr,
    Temp,
}

const SENIOR_FIRST_FLOW: &[FlowStep] = &[
    FlowStep::Key("Tab"),
    FlowStep::Delay(300),
    FlowStep::Key("Enter"),
    FlowStep::Delay(500),
    FlowStep::Key("Down"),
    FlowStep::Delay(220),
    FlowStep::Key("Enter"),
    FlowStep::Delay(650),
    FlowStep::Vital(VitalField::Sbp),
    FlowStep::Vital(VitalField::Dbp),
    FlowStep::Vital(VitalField::Hr),
    FlowStep::Vital(VitalField::Rr),
    FlowStep::Vital(VitalField::Temp),
    FlowStep::Key("Tab"),
    FlowStep::Delay(450),
    FlowStep::Phrase("bq2"),
    FlowStep::Key("Tab"),
    FlowStep::Phrase("zs2"),
    FlowStep::Key("Tab"),
    FlowStep::Phrase("jws1"),
    FlowStep::Key("Tab"),
    FlowStep::Phrase("ct2"),
    FlowStep::Key("Tab"),
    FlowStep::Phrase("jc2"),
    FlowStep::Key("Tab"),
    FlowStep::Key("Tab"),
    FlowStep::Phrase("td2"),
    FlowStep::Key("Tab"),
    FlowStep::Phrase("zd2"),
    FlowStep::Key("Tab"),
    FlowStep::Phrase("yb2"),
    FlowStep::Key("Tab"),
    FlowStep::Placeholder(PlaceholderField::Differential, ""),
    FlowStep::Key("Tab"),
    FlowStep::Key("Tab"),
    FlowStep::Placeholder(PlaceholderField::TreatmentPlan, "jh2"),
    FlowStep::Key("Tab"),
    FlowStep::Phrase("yb2"),
    FlowStep::Key("Tab"),
];

const POST_DAILY_FLOW: &[FlowStep] = &[
    FlowStep::Key("Tab"),
    FlowStep::Delay(300),
    FlowStep::Phrase("sh4"),
    FlowStep::Key("Tab"),
    FlowStep::Editable("post_daily_status"),
    FlowStep::Key("Tab"),
    FlowStep::Key("Tab"),
    FlowStep::Key("Tab"),
    FlowStep::Key("Tab"),
    FlowStep::Key("Tab"),
    FlowStep::Key("Tab"),
    FlowStep::Vital(VitalField::Temp),
    FlowStep::Vital(VitalField::Hr),
    FlowStep::Vital(VitalField::Rr),
    FlowStep::Vital(VitalField::Sbp),
    FlowStep::Vital(VitalField::Dbp),
    FlowStep::Key("Tab"),
    FlowStep::Key("Tab"),
    FlowStep::Phrase("ct3"),
    FlowStep::Key("Tab"),
    FlowStep::Phrase("jc3"),
    FlowStep::Key("Tab"),
    FlowStep::Phrase("sh5"),
];

const POST_SENIOR_FIRST_FLOW: &[FlowStep] = &[
    FlowStep::Key("Tab"),
    FlowStep::Delay(260),
    FlowStep::Key("Tab"),
    FlowStep::Delay(260),
    FlowStep::Key("Enter"),
    FlowStep::Delay(360),
    FlowStep::Key("Down"),
    FlowStep::Delay(220),
    FlowStep::Key("Enter"),
    FlowStep::Delay(500),
    FlowStep::Vital(VitalField::Sbp),
    FlowStep::Vital(VitalField::Dbp),
    FlowStep::Vital(VitalField::Hr),
    FlowStep::Vital(VitalField::Rr),
    FlowStep::Vital(VitalField::Temp),
    FlowStep::Key("Tab"),
    FlowStep::Delay(320),
    FlowStep::Editable("post_senior_summary"),
    FlowStep::Key("Tab"),
    FlowStep::Editable("post_senior_complaint"),
    FlowStep::Key("Tab"),
    FlowStep::Editable("post_senior_exam"),
    FlowStep::Key("Tab"),
    FlowStep::Editable("post_senior_tests"),
    FlowStep::Key("Tab"),
    FlowStep::Key("Tab"),
    FlowStep::Editable("post_senior_plan"),
];

const PREOP_DISCUSSION_FLOW: &[FlowStep] = &[
    FlowStep::Key("Tab"),
    FlowStep::Delay(240),
    FlowStep::RepeatKey("Enter", 2),
    FlowStep::Key("Tab"),
    FlowStep::Key("Tab"),
    FlowStep::RepeatKey("Enter", 2),
    FlowStep::RepeatKey("Tab", 6),
    FlowStep::Editable("preop_anesthesia"),
    FlowStep::Key("Tab"),
    FlowStep::Editable("preop_history"),
    FlowStep::Key("Tab"),
    FlowStep::Editable("preop_attending_speech"),
    FlowStep::Key("Tab"),
    FlowStep::Editable("preop_chief_speech"),
    FlowStep::Key("Tab"),
    FlowStep::Editable("preop_conclusion"),
    FlowStep::RepeatKey("Delete", 15),
];

fn run_builtin_flow(kind: FlowKind, options: Options, disease_index: usize) {
    if let FlowKind::FirstCourse = kind {
        run_first_course_flow(options, disease_index);
        return;
    }
    if let FlowKind::DischargeCertificate = kind {
        run_discharge_certificate_flow(disease_index);
        return;
    }

    let phrases = load_phrases();
    let fixed_texts = load_fixed_texts();
    let template_file = match kind {
        FlowKind::SeniorFirst => "senior_round.txt",
        FlowKind::PreopDiscussion => "preop_discussion.txt",
        FlowKind::PostSeniorFirst => "post_senior_first.txt",
        FlowKind::PostDaily => "post_op_daily.txt",
        _ => unreachable!(),
    };
    let template_fields = load_disease_template_fields(disease_index, template_file);
    let placeholders = load_disease_placeholders(disease_index, template_file);
    let vitals = random_vitals(options);
    let steps = match kind {
        FlowKind::FirstCourse => unreachable!(),
        FlowKind::SeniorFirst => SENIOR_FIRST_FLOW,
        FlowKind::PreopDiscussion => PREOP_DISCUSSION_FLOW,
        FlowKind::PostSeniorFirst => POST_SENIOR_FIRST_FLOW,
        FlowKind::PostDaily => POST_DAILY_FLOW,
        FlowKind::DischargeCertificate => unreachable!(),
    };

    for step in steps {
        if STOP_TYPING.load(Ordering::SeqCst) {
            return;
        }

        match *step {
            FlowStep::Delay(ms) => template_sleep(ms),
            FlowStep::Key(key) => press_named_key(key),
            FlowStep::RepeatKey(key, count) => {
                for _ in 0..count {
                    press_named_key(key);
                    template_sleep(120);
                }
            }
            FlowStep::Editable(key) => {
                if let Some(text) = field_or_fallback(&template_fields, key, fixed_texts.get(key)) {
                    type_text(text);
                } else {
                    log_event(&format!("固定文本缺少键：{}", key));
                }
            }
            FlowStep::Phrase(code) => {
                if let Some(text) = field_or_fallback(&template_fields, code, phrases.get(code)) {
                    type_text(text);
                } else {
                    log_event(&format!("短语集缺少键：{}", code));
                }
            }
            FlowStep::Placeholder(field, fallback_phrase) => {
                let text = placeholder_text(&placeholders, field);
                if !text.is_empty() {
                    type_text(text);
                } else if !fallback_phrase.is_empty() {
                    if let Some(text) = phrases.get(fallback_phrase) {
                        type_text(text);
                    } else {
                        log_event(&format!("短语集缺少占位符回退键：{}", fallback_phrase));
                    }
                }
            }
            FlowStep::Vital(field) => enter_vital_field(vitals, field),
        }

        template_sleep(130);
    }
}

fn run_discharge_certificate_flow(disease_index: usize) {
    let advice = discharge_advice_text(disease_index);

    press_named_key("Tab");
    template_sleep(520);
    press_named_key("Enter");
    template_sleep(520);
    press_named_key("Enter");
    template_sleep(620);
    press_named_key("Down");
    template_sleep(420);
    type_text(discharge_certificate_diagnosis_marker());
    template_sleep(260);

    for _ in 0..3 {
        press_named_key("Down");
        template_sleep(180);
    }

    press_named_key("Enter");
    template_sleep(220);
    type_text(&advice);
    template_sleep(260);

    press_named_key("Right");
    template_sleep(180);
    press_shift_repeat_key("Down", 5);
    template_sleep(180);
    press_named_key("Delete");
    template_sleep(240);

    for _ in 0..4 {
        press_named_key("Tab");
        template_sleep(220);
        press_named_key("Enter");
        template_sleep(180);
        press_named_key("Enter");
        template_sleep(240);
    }

    press_named_key("Tab");
    template_sleep(220);
    press_named_key("Enter");
    template_sleep(180);
    press_named_key("Enter");
    template_sleep(240);
    press_named_key("Enter");
    template_sleep(180);
    press_named_key("Enter");
    template_sleep(240);
}

fn discharge_certificate_diagnosis_marker() -> &'static str {
    " "
}

fn run_first_course_flow(options: Options, disease_index: usize) {
    let vitals = random_vitals(options);
    let template_fields = load_disease_template_fields(disease_index, "first_visit.txt");
    let placeholders = load_disease_placeholders(disease_index, "first_visit.txt");
    let fixed_texts = load_fixed_texts();
    let differential_items = numbered_items(&placeholders.differential);
    let treatment_plan_items = numbered_items(&placeholders.treatment_plan);

    press_named_key("Tab");
    template_sleep(480);
    press_named_key("Enter");
    template_sleep(520);
    press_named_key("Down");
    template_sleep(220);
    press_named_key("Down");
    template_sleep(260);
    press_named_key("Enter");
    template_sleep(360);
    press_named_key("Tab");
    template_sleep(260);
    if let Some(text) = field_or_fallback(
        &template_fields,
        "first_course_type",
        fixed_texts.get("first_course_type"),
    ) {
        type_text(text);
    }
    template_sleep(260);

    press_named_key("Tab");
    template_sleep(240);
    type_text(&vital_text(vitals, VitalField::Temp));
    template_sleep(160);
    press_named_key("Tab");
    template_sleep(160);
    type_text(&vital_text(vitals, VitalField::Hr));
    template_sleep(160);
    press_named_key("Tab");
    template_sleep(160);
    type_text(&vital_text(vitals, VitalField::Rr));
    template_sleep(160);
    press_named_key("Tab");
    template_sleep(160);
    type_text(&vital_text(vitals, VitalField::Sbp));
    template_sleep(160);
    press_named_key("Tab");
    template_sleep(160);
    type_text(&vital_text(vitals, VitalField::Dbp));
    template_sleep(200);

    press_named_key("Tab");
    template_sleep(240);
    press_named_key("Up");
    template_sleep(160);
    press_named_key("Up");
    template_sleep(180);
    press_named_key("Enter");
    template_sleep(240);
    if differential_items.is_empty() {
        if let Some(text) = field_or_fallback(
            &template_fields,
            "first_course_differential_fallback",
            fixed_texts.get("first_course_differential_fallback"),
        ) {
            type_text(text);
        }
    } else {
        type_numbered_items(&differential_items);
    }
    template_sleep(260);

    press_named_key("Down");
    template_sleep(220);
    press_named_key("Enter");
    template_sleep(240);
    if treatment_plan_items.is_empty() {
        if let Some(text) = field_or_fallback(
            &template_fields,
            "first_course_plan_fallback",
            fixed_texts.get("first_course_plan_fallback"),
        ) {
            type_text(text);
        }
    } else {
        type_numbered_items(&treatment_plan_items);
    }
    template_sleep(260);
    press_alt_combo(0x44);
}

fn numbered_items(text: &str) -> Vec<String> {
    let text = text.trim();
    if text.is_empty() {
        return Vec::new();
    }
    let mut starts = Vec::new();
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    for i in 0..chars.len().saturating_sub(1) {
        let ch = chars[i].1;
        let next = chars[i + 1].1;
        let boundary = i == 0
            || matches!(
                chars[i.saturating_sub(1)].1,
                '；' | ';' | '。' | '\n' | '\r'
            );
        if boundary && ch.is_ascii_digit() && next == '、' {
            starts.push(chars[i].0);
        }
    }
    if starts.len() <= 1 {
        return vec![text.to_string()];
    }
    let mut items = Vec::new();
    for (idx, start) in starts.iter().enumerate() {
        let end = starts.get(idx + 1).copied().unwrap_or(text.len());
        let item = text[*start..end]
            .trim()
            .trim_end_matches('；')
            .trim_end_matches(';')
            .trim()
            .to_string();
        if !item.is_empty() {
            items.push(item);
        }
    }
    items
}

fn type_numbered_items(items: &[String]) {
    for (idx, item) in items.iter().enumerate() {
        if STOP_TYPING.load(Ordering::SeqCst) {
            return;
        }
        type_text(item);
        if idx + 1 < items.len() {
            template_sleep(160);
            press_named_key("Enter");
            template_sleep(180);
        }
    }
}

fn discharge_advice_text(disease_index: usize) -> String {
    let fields = load_disease_template_fields(disease_index, "discharge_certificate.txt");
    let text = fields
        .get("discharge_advice_default")
        .cloned()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| load_disease_template(disease_index, "discharge_certificate.txt"));
    let text = text.trim();
    if text.is_empty() || text.contains("待完善") {
        load_fixed_texts()
            .remove("discharge_advice_default")
            .unwrap_or_default()
    } else {
        text.to_string()
    }
}

fn vital_text(vitals: Vitals, field: VitalField) -> String {
    match field {
        VitalField::Sbp => vitals.sbp.to_string(),
        VitalField::Dbp => vitals.dbp.to_string(),
        VitalField::Hr => vitals.hr.to_string(),
        VitalField::Rr => vitals.rr.to_string(),
        VitalField::Temp => format!("{}.{}", vitals.temp10 / 10, vitals.temp10 % 10),
    }
}

fn enter_vital_field(vitals: Vitals, field: VitalField) {
    press_named_key("Tab");
    template_sleep(360);
    press_named_key("Enter");
    template_sleep(520);
    type_text(&vital_text(vitals, field));
    template_sleep(260);
    press_named_key("Enter");
    template_sleep(500);
}

#[derive(Clone, Copy)]
enum OpenLaunchTarget {
    Medical,
    Order,
}

unsafe fn queue_open_program_flow(username: String, password: String, target: OpenLaunchTarget) {
    match target {
        OpenLaunchTarget::Medical => PENDING_MEDICAL_LAUNCH.store(true, Ordering::SeqCst),
        OpenLaunchTarget::Order => PENDING_ORDER_LAUNCH.store(true, Ordering::SeqCst),
    }
    STOP_TYPING.store(false, Ordering::SeqCst);
    set_status("已加入启动队列，1.5 秒后执行...");

    if OPEN_PROGRAM_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    thread::spawn(move || {
        loop {
            sleep_interruptible(1500);
            let run_medical = PENDING_MEDICAL_LAUNCH.swap(false, Ordering::SeqCst);
            let run_order = PENDING_ORDER_LAUNCH.swap(false, Ordering::SeqCst);
            if !run_medical && !run_order {
                break;
            }

            if run_order {
                run_order_system_flow(username.clone(), password.clone());
                if run_medical {
                    sleep_interruptible(2000);
                }
            }
            if run_medical {
                run_medical_system_flow(username.clone());
            }

            if !PENDING_MEDICAL_LAUNCH.load(Ordering::SeqCst)
                && !PENDING_ORDER_LAUNCH.load(Ordering::SeqCst)
            {
                let settings = load_settings();
                if settings.auto_remote_sign {
                    unsafe { start_remote_sign_flow(settings) };
                }
                break;
            }
        }
        OPEN_PROGRAM_RUNNING.store(false, Ordering::SeqCst);
    });
}

unsafe fn start_remote_sign_flow(settings: Settings) {
    if REMOTE_SIGN_RUNNING.swap(true, Ordering::SeqCst) {
        set_status("服务端一键签名请求正在进行中...");
        return;
    }

    let host = configured_remote_sign_server(&settings);
    let port = configured_remote_sign_port(&settings);
    set_status(&format!("正在请求服务端 {}:{} 启动一键签名...", host, port));
    thread::spawn(move || {
        let result = request_remote_signature_launch(&host, port);
        REMOTE_SIGN_RUNNING.store(false, Ordering::SeqCst);
        unsafe {
            match result {
                Ok(message) => {
                    log_event(&format!("服务端一键签名已启动：{}", message));
                    set_status("服务端一键签名已启动。");
                }
                Err(error) => {
                    log_event(&format!("服务端一键签名失败：{}", error));
                    set_status(&format!("服务端一键签名失败：{}", error));
                }
            }
        }
    });
}

fn request_remote_signature_launch(host: &str, port: u16) -> Result<String, String> {
    let endpoint = format!("{}:{}", host.trim(), port);
    let address: SocketAddr = endpoint
        .to_socket_addrs()
        .map_err(|error| format!("服务器地址无效：{}", error))?
        .next()
        .ok_or_else(|| "服务器地址未解析到可用 IP。".to_string())?;
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(3))
        .map_err(|error| format!("无法连接服务端：{}", error))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| format!("设置服务端读取超时失败：{}", error))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .map_err(|error| format!("设置服务端写入超时失败：{}", error))?;
    stream
        .write_all(b"RUN_SIGNATURE\n")
        .map_err(|error| format!("发送服务端请求失败：{}", error))?;
    let mut response = [0u8; 256];
    let length = stream
        .read(&mut response)
        .map_err(|error| format!("读取服务端响应失败：{}", error))?;
    let message = String::from_utf8_lossy(&response[..length])
        .trim()
        .to_string();
    if message.starts_with("OK") {
        Ok(message)
    } else if message.is_empty() {
        Err("服务端未返回确认消息。".to_string())
    } else {
        Err(message)
    }
}

fn run_medical_system_flow(username: String) {
    let username = if username.trim().is_empty() {
        "Y05275".to_string()
    } else {
        username
    };
    let points = launch_points();

    spawn_with_program_directory(&configured_medical_system_path());
    sleep_interruptible(4000);
    mouse_move(points.medical_focus.0, points.medical_focus.1);
    mouse_left_click();
    sleep_interruptible(500);
    if load_settings().launch_alt_f4 {
        unsafe {
            send_virtual_key(0x12, false);
            send_virtual_key(0x73, false);
            send_virtual_key(0x73, true);
            send_virtual_key(0x12, true);
        }
        sleep_interruptible(600);
    }
    click_login_checkbox_by_name("其他登录方式", None, 2500);
    sleep_interruptible(500);
    mouse_move(points.medical_login.0, points.medical_login.1);
    mouse_left_click();
    sleep_interruptible(250);
    type_literal_text(&username);
    sleep_interruptible(120);
    press_named_key("Enter");
    sleep_interruptible(150);
    press_named_key("Enter");
    sleep_interruptible(1200);
}

fn run_order_system_flow(username: String, password: String) {
    let username = if username.trim().is_empty() {
        "Y05275".to_string()
    } else {
        username
    };
    let password = if password.is_empty() {
        "Aa123456".to_string()
    } else {
        password
    };

    spawn_with_program_directory(&configured_order_system_path());
    sleep_interruptible(1200);
    type_literal_text(&username);
    sleep_interruptible(120);
    press_named_key("Tab");
    sleep_interruptible(120);
    type_literal_text(&password);
    sleep_interruptible(1300);
    click_login_checkbox_by_name("扫码签名", None, 2500);
    sleep_interruptible(180);
    click_login_button_by_name("确定", None, 2500);
}

fn click_login_checkbox_by_name(name: &str, fallback: Option<(i32, i32)>, timeout_ms: u64) -> bool {
    if unsafe { click_visible_checkbox_by_uia(name, timeout_ms) }.is_ok() {
        log_event(&format!("登录流程已通过控件查找点击复选框：{}", name));
        return true;
    }
    if let Some((x, y)) = fallback {
        log_event(&format!(
            "登录流程未找到复选框“{}”，使用分辨率点位兜底：({}, {})",
            name, x, y
        ));
        mouse_move(x, y);
        mouse_left_click();
        return true;
    }
    log_event(&format!("登录流程未找到复选框“{}”，且没有兜底点位", name));
    false
}

fn click_login_button_by_name(name: &str, fallback: Option<(i32, i32)>, timeout_ms: u64) -> bool {
    if unsafe { click_visible_button_by_uia(name, timeout_ms) }.is_ok() {
        log_event(&format!("登录流程已通过控件查找调用按钮：{}", name));
        return true;
    }
    if let Some((x, y)) = fallback {
        log_event(&format!(
            "登录流程未找到按钮“{}”，使用分辨率点位兜底：({}, {})",
            name, x, y
        ));
        mouse_move(x, y);
        mouse_left_click();
        return true;
    }
    log_event(&format!("登录流程未找到按钮“{}”，且没有兜底点位", name));
    false
}

unsafe fn click_visible_checkbox_by_uia(name: &str, timeout_ms: u64) -> Result<(), String> {
    CoInitializeEx(None, COINIT_APARTMENTTHREADED)
        .ok()
        .map_err(|error| format!("初始化UIA失败：{}", error))?;
    let result = (|| {
        let automation: IUIAutomation =
            CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
                .map_err(|error| format!("创建UIA失败：{}", error))?;
        let root = automation
            .GetRootElement()
            .map_err(|error| format!("获取桌面根控件失败：{}", error))?;
        let checkbox = wait_for_named_control(
            &automation,
            &root,
            name,
            UIA_CheckBoxControlTypeId.0,
            timeout_ms,
        )?;
        let rect = checkbox
            .CurrentBoundingRectangle()
            .map_err(|error| format!("读取复选框“{}”位置失败：{}", name, error))?;
        if rect.right <= rect.left || rect.bottom <= rect.top {
            return Err(format!("复选框“{}”没有有效屏幕位置", name));
        }
        let x = rect.left + (rect.right - rect.left) / 2;
        let y = rect.top + (rect.bottom - rect.top) / 2;
        let _ = checkbox.SetFocus();
        SetCursorPos(x, y);
        course_sleep(100);
        mouse_left_click();
        Ok(())
    })();
    CoUninitialize();
    result
}

unsafe fn click_visible_button_by_uia(name: &str, timeout_ms: u64) -> Result<(), String> {
    CoInitializeEx(None, COINIT_APARTMENTTHREADED)
        .ok()
        .map_err(|error| format!("初始化UIA失败：{}", error))?;
    let result = (|| {
        let automation: IUIAutomation =
            CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
                .map_err(|error| format!("创建UIA失败：{}", error))?;
        let root = automation
            .GetRootElement()
            .map_err(|error| format!("获取桌面根控件失败：{}", error))?;
        let button = wait_for_named_control(
            &automation,
            &root,
            name,
            UIA_ButtonControlTypeId.0,
            timeout_ms,
        )?;
        let rect = button
            .CurrentBoundingRectangle()
            .map_err(|error| format!("读取按钮“{}”位置失败：{}", name, error))?;
        if rect.right <= rect.left || rect.bottom <= rect.top {
            return Err(format!("按钮“{}”没有有效屏幕位置", name));
        }
        let x = rect.left + (rect.right - rect.left) / 2;
        let y = rect.top + (rect.bottom - rect.top) / 2;
        let _ = button.SetFocus();
        SetCursorPos(x, y);
        course_sleep(80);
        if let Ok(invoke) =
            button.GetCurrentPatternAs::<IUIAutomationInvokePattern>(UIA_InvokePatternId)
        {
            invoke
                .Invoke()
                .map_err(|error| format!("调用按钮“{}”失败：{}", name, error))?;
        } else {
            mouse_left_click();
        }
        Ok(())
    })();
    CoUninitialize();
    result
}

fn default_medical_system_path() -> &'static str {
    r"C:\JHEMR\JHEMRCentral.Win.exe"
}

fn default_order_system_path() -> &'static str {
    r"D:\doctor\doctws.exe"
}

fn default_nursing_system_path() -> &'static str {
    r"D:\cmis-6.0\bin\MinimaxNIS.exe"
}

fn setting_or_default(value: &str, default_value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        default_value.to_string()
    } else {
        trimmed.to_string()
    }
}

fn configured_medical_system_path() -> String {
    let settings = load_settings();
    setting_or_default(&settings.medical_system_path, default_medical_system_path())
}

fn configured_order_system_path() -> String {
    let settings = load_settings();
    setting_or_default(&settings.order_system_path, default_order_system_path())
}

fn configured_nursing_system_path() -> String {
    let settings = load_settings();
    setting_or_default(&settings.nursing_system_path, default_nursing_system_path())
}

fn configured_remote_sign_server(settings: &Settings) -> String {
    setting_or_default(&settings.remote_sign_server, DEFAULT_REMOTE_SIGN_SERVER)
}

fn configured_remote_sign_port(settings: &Settings) -> u16 {
    if settings.remote_sign_port == 0 {
        DEFAULT_REMOTE_SIGN_PORT
    } else {
        settings.remote_sign_port
    }
}

fn normalized_remote_sign_port(value: &str) -> u16 {
    value
        .trim()
        .parse::<u16>()
        .ok()
        .filter(|port| *port > 0)
        .unwrap_or(DEFAULT_REMOTE_SIGN_PORT)
}

fn launch_points() -> LaunchPoints {
    match LAUNCH_RESOLUTION_PROFILE.load(Ordering::SeqCst).min(1) {
        1 => launch_points_2160(),
        _ => launch_points_1080(),
    }
}

fn launch_points_1080() -> LaunchPoints {
    LaunchPoints {
        medical_focus: (1038, 492),
        medical_login: (1023, 677),
    }
}

fn launch_points_2160() -> LaunchPoints {
    LaunchPoints {
        medical_focus: (1255, 485),
        medical_login: (1433, 316),
    }
}

unsafe fn start_cleanup_running_apps() {
    if CLEANUP_RUNNING.swap(true, Ordering::SeqCst) {
        set_status("正在清理已运行程序...");
        return;
    }
    set_status("正在清理病历、住院医生站、Edge 和 Chrome...");
    thread::spawn(|| {
        let requested = cleanup_running_apps();
        CLEANUP_RUNNING.store(false, Ordering::SeqCst);
        unsafe {
            set_status(&format!("已发送 {} 项程序清理请求。", requested));
        }
    });
}

fn cleanup_running_apps() -> usize {
    let mut image_names = vec![
        "msedge.exe".to_string(),
        "chrome.exe".to_string(),
        "JHEMRCentral.Win.exe".to_string(),
        "doctws.exe".to_string(),
        "EMR-Release.exe".to_string(),
    ];
    for path in [
        configured_medical_system_path(),
        configured_order_system_path(),
    ] {
        if let Some(name) = Path::new(&path)
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.trim().is_empty())
        {
            if !image_names
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(name))
            {
                image_names.push(name.to_string());
            }
        }
    }

    let mut requested = 0;
    for image_name in image_names {
        let _ = Command::new("taskkill")
            .args(["/F", "/T", "/IM", &image_name])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
        requested += 1;
    }
    for window_title in ["EMR-Release 增加登录类型*", "卫生部北京医院住院医生站系统*"]
    {
        let filter = format!("WINDOWTITLE eq {}", window_title);
        let _ = Command::new("taskkill")
            .args(["/F", "/T", "/FI", &filter])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
        requested += 1;
    }
    log_event("已执行运行程序清理：病历系统、住院医生站、Edge、Chrome。");
    requested
}

unsafe fn start_nursing_flow(account: String) {
    STOP_TYPING.store(false, Ordering::SeqCst);
    set_status("正在启动远卓护理，约 5 秒后自动输入账号密码...");
    thread::spawn(move || {
        run_nursing_flow(account);
    });
}

fn run_nursing_flow(account: String) {
    let _ = Command::new("taskkill")
        .args(["/IM", "MinimaxNIS.exe", "/F"])
        .status();
    sleep_interruptible(600);
    spawn_with_program_directory(&configured_nursing_system_path());
    sleep_interruptible(4000);
    let screen_x = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let screen_y = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    mouse_move(screen_x / 2, screen_y * 2 / 3);
    mouse_left_click();
    sleep_interruptible(1000);
    press_named_key("Tab");
    sleep_interruptible(180);
    type_literal_text(&account);
    sleep_interruptible(180);
    press_named_key("Tab");
    sleep_interruptible(180);
    type_literal_text(&account);
    sleep_interruptible(180);
    press_named_key("Enter");
}

unsafe fn start_save_order_flow() {
    if SAVE_ORDER_RUNNING.swap(true, Ordering::SeqCst) {
        set_status("医嘱自动保存正在执行。");
        return;
    }
    STOP_TYPING.store(false, Ordering::SeqCst);
    set_status("正在执行医嘱自动保存...");
    thread::spawn(|| {
        run_save_order_flow();
        SAVE_ORDER_RUNNING.store(false, Ordering::SeqCst);
    });
}

fn run_save_order_flow() {
    press_alt_combo(0x53);
    sleep_interruptible(350);
    press_alt_combo(0x54);
    sleep_interruptible(350);
    press_alt_combo(0x59);
    sleep_interruptible(350);
    press_named_key("Enter");
}

unsafe fn show_vte_stall_alert() {
    if VTE_ALERT_SHOWN.swap(true, Ordering::SeqCst) {
        return;
    }
    STOP_TYPING.store(true, Ordering::SeqCst);
    set_status("入院记录创建已停止；请确认是否完成VTE评估。");
    log_event("入院记录创建超时或失败，已终止流程并提示确认VTE评估");
    MessageBoxW(
        APP.hwnd,
        wide("请确认是否完成VTE评估").as_ptr(),
        wide("入院记录创建提醒").as_ptr(),
        MB_OK | MB_ICONWARNING,
    );
}

unsafe fn start_create_courses_flow(
    kind: CourseBatchKind,
    base_time: String,
    attending_superior: String,
    chief_superior: String,
) {
    if CREATE_ALL_COURSES_RUNNING.swap(true, Ordering::SeqCst) {
        set_status("一键创建病程正在执行。");
        log_event("重复点击被忽略：一键创建病程已经在运行");
        return;
    }
    let run_id = COURSE_RUN_ID
        .fetch_add(1, Ordering::SeqCst)
        .saturating_add(1);
    CURRENT_COURSE_ITEM.store(0, Ordering::SeqCst);
    VTE_ALERT_SHOWN.store(false, Ordering::SeqCst);
    let (batch_label, count) = match kind {
        CourseBatchKind::Preop => ("术前病程", PREOP_COURSE_SPECS.len()),
        CourseBatchKind::Postop => ("术后病程", POSTOP_COURSE_SPECS.len()),
    };
    log_event(&format!(
        "========== 一键创建{}开始，基准时间={}，版本={} ==========",
        batch_label, base_time, APP_VERSION
    ));
    log_event(&format!(
        "本次上级医师：主治={}，主任={}",
        attending_superior, chief_superior
    ));
    let (base_date, _) = match parse_base_datetime(&base_time) {
        Ok(value) => value,
        Err(error) => {
            CREATE_ALL_COURSES_RUNNING.store(false, Ordering::SeqCst);
            set_status(&format!("{}时间格式错误：{}", batch_label, error));
            log_event(&format!("{}时间格式错误：{}", batch_label, error));
            return;
        }
    };
    STOP_TYPING.store(false, Ordering::SeqCst);
    set_status(&format!("约1秒后开始创建{}份{}。", count, batch_label));
    if matches!(kind, CourseBatchKind::Preop) {
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(18));
            if COURSE_RUN_ID.load(Ordering::SeqCst) == run_id
                && CREATE_ALL_COURSES_RUNNING.load(Ordering::SeqCst)
                && CURRENT_COURSE_ITEM.load(Ordering::SeqCst) == 1
            {
                unsafe {
                    show_vte_stall_alert();
                }
            }
        });
    }
    thread::spawn(move || {
        sleep_interruptible(1000);
        let result = if STOP_TYPING.load(Ordering::SeqCst) {
            Ok(())
        } else {
            run_create_courses(kind, base_date, attending_superior, chief_superior)
        };
        let failed_on_admission = result.is_err()
            && matches!(kind, CourseBatchKind::Preop)
            && CURRENT_COURSE_ITEM.load(Ordering::SeqCst) == 1;
        if failed_on_admission {
            unsafe {
                show_vte_stall_alert();
            }
        }
        CURRENT_COURSE_ITEM.store(0, Ordering::SeqCst);
        CREATE_ALL_COURSES_RUNNING.store(false, Ordering::SeqCst);
        unsafe {
            if STOP_TYPING.load(Ordering::SeqCst) {
                set_status("一键创建病程已停止。");
                log_event("流程被右Ctrl+右Alt手动停止");
            } else if let Err(error) = result {
                set_status(&format!("一键创建病程失败：{}", error));
                log_event(&format!("流程失败：{}", error));
            } else {
                set_status(&format!("{}份{}创建流程已完成。", count, batch_label));
                log_event(&format!("{}份{}创建流程完成", count, batch_label));
            }
        }
    });
}

fn run_create_courses(
    kind: CourseBatchKind,
    base_date: SimpleDate,
    attending_superior: String,
    chief_superior: String,
) -> Result<(), String> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .map_err(|error| format!("初始化UI Automation失败：{}", error))?;

        let result = (|| {
            let automation: IUIAutomation =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
                    .map_err(|error| format!("创建UI Automation失败：{}", error))?;
            let root = automation
                .GetRootElement()
                .map_err(|error| format!("读取桌面控件树失败：{}", error))?;
            let emr = find_emr_window(&automation, &root)?;
            let mut config = load_create_courses_config();
            if !attending_superior.trim().is_empty() {
                config.attending_superior = attending_superior.trim().to_string();
            }
            if !chief_superior.trim().is_empty() {
                config.chief_superior = chief_superior.trim().to_string();
            }

            let specs: &[CourseCreateSpec] = match kind {
                CourseBatchKind::Preop => &PREOP_COURSE_SPECS,
                CourseBatchKind::Postop => &POSTOP_COURSE_SPECS,
            };
            for (index, spec) in specs.iter().enumerate() {
                if STOP_TYPING.load(Ordering::SeqCst) {
                    break;
                }
                CURRENT_COURSE_ITEM.store(index + 1, Ordering::SeqCst);
                set_status(&format!(
                    "正在创建 {}/{}：{}",
                    index + 1,
                    specs.len(),
                    spec.template
                ));
                log_event(&format!(
                    "开始第{}/{}项：{}，父目录={}，日期偏移={}天，时间={}",
                    index + 1,
                    specs.len(),
                    spec.template,
                    spec.parent,
                    spec.day_offset,
                    spec.clock
                ));
                create_one_course(&automation, &root, &emr, *spec, base_date, &config)
                    .map_err(|error| format!("第{}项“{}”：{}", index + 1, spec.template, error))?;
                log_event(&format!(
                    "第{}/{}项完成：{}",
                    index + 1,
                    specs.len(),
                    spec.template
                ));
                CURRENT_COURSE_ITEM.store(0, Ordering::SeqCst);
            }
            Ok(())
        })();

        CoUninitialize();
        result
    }
}

#[derive(Default)]
struct CreateCoursesConfig {
    attending_superior: String,
    chief_superior: String,
}

fn create_one_course(
    automation: &IUIAutomation,
    root: &IUIAutomationElement,
    emr: &IUIAutomationElement,
    spec: CourseCreateSpec,
    base_date: SimpleDate,
    config: &CreateCoursesConfig,
) -> Result<(), String> {
    unsafe {
        let parent =
            find_last_named_control(automation, emr, spec.parent, UIA_TreeItemControlTypeId.0)
                .map_err(|_| format!("未找到主目录“{}”", spec.parent))?;
        select_tree_item(&parent, spec.parent)?;
        log_event(&format!("已选中父目录“{}”，准备发送F4", spec.parent));
        press_named_key("F4");

        let template_window = wait_for_template_window(automation, 6000)?;
        log_event("已找到“模板列表”窗口");
        let template = find_template_item(automation, &template_window, spec.template)?;
        select_tree_item(&template, spec.template)?;
        log_event(&format!("已选中模板“{}”", spec.template));
        course_sleep(140);

        if let Some(document_title) = resolve_course_document_title(spec, config) {
            set_template_document_title(automation, &template_window, &document_title)?;
            log_event(&format!("文档标题已设置并验证：{}", document_title));
        }

        let date = add_days(base_date, spec.day_offset);
        let clock = spec.clock;
        let datetime = format!(
            "{:04}-{:02}-{:02} {}",
            date.year, date.month, date.day, clock
        );
        set_template_datetime(automation, &template_window, &datetime, date, clock)?;
        log_event(&format!("模板时间已设置并验证：{}", datetime));

        if let Some(superior_kind) = spec.superior_kind {
            let superior = match superior_kind {
                SuperiorKind::Attending => config.attending_superior.trim(),
                SuperiorKind::Chief => config.chief_superior.trim(),
            };
            if superior.is_empty() {
                return Err("create_courses_config.txt中的上级医师为空".to_string());
            }
            set_superior(automation, root, &template_window, superior)?;
        }

        log_event("准备非阻塞点击模板列表右下角“确定”");
        click_named_button_nonblocking(automation, &template_window, "确定")?;
        log_event("模板列表“确定”已完成鼠标点击");
        if spec.template == "神经外科入院记录二" {
            set_status("入院记录：正在确认过敏史...");
            log_event("模板确定点击完成；立即发送Enter确认过敏史，不使用固定等待");
            press_named_key("Enter");
        }
        log_event("等待模板列表关闭及可选提示窗口");
        wait_for_course_creation_completion(automation, root, 6000)?;
        log_event("已确认模板列表关闭");
        course_sleep(260);
        Ok(())
    }
}

fn resolve_course_document_title(
    spec: CourseCreateSpec,
    config: &CreateCoursesConfig,
) -> Option<String> {
    match spec.document_title {
        CourseDocumentTitle::Default => None,
        CourseDocumentTitle::PostopDayOneSurgeon => Some(format!(
            "术后第1日{}术者查房记录",
            config.chief_superior.trim()
        )),
        CourseDocumentTitle::Fixed(title) => Some(title.to_string()),
    }
}

unsafe fn confirm_dialog_with_fallback(
    automation: &IUIAutomation,
    _root: &IUIAutomationElement,
    dialog: &IUIAutomationElement,
    dialog_name: &str,
) -> Result<(), String> {
    let button = find_last_named_control(automation, dialog, "确定", UIA_ButtonControlTypeId.0)?;
    let rect = button
        .CurrentBoundingRectangle()
        .map_err(|error| format!("读取“{}”确定按钮位置失败：{}", dialog_name, error))?;
    let x = rect.left + (rect.right - rect.left) / 2;
    let y = rect.top + (rect.bottom - rect.top) / 2;

    let mut title_hwnd = find_top_level_window(dialog_name);
    for _ in 0..5 {
        if title_hwnd != 0 {
            break;
        }
        thread::sleep(Duration::from_millis(60));
        title_hwnd = find_top_level_window(dialog_name);
    }
    let dialog_hwnd = if title_hwnd != 0 {
        title_hwnd
    } else {
        dialog
            .CurrentNativeWindowHandle()
            .ok()
            .filter(|hwnd| !hwnd.0.is_null())
            .map(|hwnd| hwnd.0 as Hwnd)
            .unwrap_or(0)
    };
    let track_by_title = title_hwnd != 0;
    let button_hwnd = button
        .CurrentNativeWindowHandle()
        .ok()
        .filter(|hwnd| !hwnd.0.is_null())
        .map(|hwnd| hwnd.0 as Hwnd)
        .unwrap_or(0);

    log_event(&format!(
        "弹窗“{}”：title_hwnd=0x{:X}，dialog_hwnd=0x{:X}，button_hwnd=0x{:X}，按钮中心=({}, {})",
        dialog_name, title_hwnd, dialog_hwnd, button_hwnd, x, y
    ));
    SetCursorPos(x, y);

    for key in ["Enter", "Space", "Enter"] {
        if dialog_hwnd != 0 {
            force_foreground_window(dialog_hwnd);
        }
        let _ = dialog.SetFocus();
        course_sleep(220);
        SetCursorPos(x, y);
        log_event(&format!("弹窗“{}”：发送{}", dialog_name, key));
        press_named_key(key);
        if wait_for_dialog_to_close(dialog_name, dialog_hwnd, track_by_title, 650) {
            log_event(&format!("弹窗“{}”已在发送{}后关闭", dialog_name, key));
            return Ok(());
        }
    }

    log_event(&format!(
        "弹窗“{}”：移动鼠标并执行可见左键点击",
        dialog_name
    ));
    SetCursorPos(x, y);
    course_sleep(120);
    mouse_left_click();
    if wait_for_dialog_to_close(dialog_name, dialog_hwnd, track_by_title, 1200) {
        log_event(&format!("弹窗“{}”已在鼠标点击后关闭", dialog_name));
        return Ok(());
    }

    if button_hwnd != 0 {
        log_event(&format!("弹窗“{}”：发送非阻塞BM_CLICK", dialog_name));
        PostMessageW(button_hwnd, BM_CLICK, 0, 0);
    }
    if dialog_hwnd != 0 {
        PostMessageW(dialog_hwnd, WM_KEYDOWN, 0x0D, 1);
        PostMessageW(dialog_hwnd, WM_KEYUP, 0x0D, 0xC0000001_u32 as Lparam);
    }
    if wait_for_dialog_to_close(dialog_name, dialog_hwnd, track_by_title, 1500) {
        log_event(&format!("弹窗“{}”已在非阻塞消息后关闭", dialog_name));
        return Ok(());
    }

    let error = format!("“{}”窗口仍然存在，确认流程已停止", dialog_name);
    log_event(&error);
    Err(error)
}

unsafe fn wait_for_dialog_to_close(
    dialog_name: &str,
    dialog_hwnd: Hwnd,
    track_by_title: bool,
    timeout_ms: u64,
) -> bool {
    let mut waited = 0;
    while waited <= timeout_ms {
        if track_by_title {
            let current = find_top_level_window(dialog_name);
            if current == 0 || IsWindowVisible(current) == 0 {
                return true;
            }
        } else if dialog_hwnd != 0 {
            if IsWindow(dialog_hwnd) == 0 || IsWindowVisible(dialog_hwnd) == 0 {
                return true;
            }
        }
        thread::sleep(Duration::from_millis(50));
        waited += 50;
    }
    false
}

unsafe fn find_top_level_window(title: &str) -> Hwnd {
    let title = wide(title);
    FindWindowW(null(), title.as_ptr())
}

unsafe fn force_foreground_window(hwnd: Hwnd) {
    if hwnd == 0 || IsWindow(hwnd) == 0 {
        return;
    }

    let current_thread = GetCurrentThreadId();
    let target_thread = GetWindowThreadProcessId(hwnd, null_mut());
    let foreground = GetForegroundWindow();
    let foreground_thread = if foreground != 0 {
        GetWindowThreadProcessId(foreground, null_mut())
    } else {
        0
    };

    let attached_target = target_thread != 0
        && target_thread != current_thread
        && AttachThreadInput(current_thread, target_thread, 1) != 0;
    let attached_foreground = foreground_thread != 0
        && foreground_thread != current_thread
        && foreground_thread != target_thread
        && AttachThreadInput(current_thread, foreground_thread, 1) != 0;

    ShowWindow(hwnd, SW_RESTORE);
    BringWindowToTop(hwnd);
    SetForegroundWindow(hwnd);
    SetActiveWindow(hwnd);
    SetFocus(hwnd);

    if attached_foreground {
        AttachThreadInput(current_thread, foreground_thread, 0);
    }
    if attached_target {
        AttachThreadInput(current_thread, target_thread, 0);
    }
}

unsafe fn restore_clipboard_target_window(hwnd: Hwnd) {
    if hwnd == 0 || IsWindow(hwnd) == 0 {
        return;
    }

    let current_thread = GetCurrentThreadId();
    let target_thread = GetWindowThreadProcessId(hwnd, null_mut());
    let foreground = GetForegroundWindow();
    let foreground_thread = if foreground != 0 {
        GetWindowThreadProcessId(foreground, null_mut())
    } else {
        0
    };

    let attached_target = target_thread != 0
        && target_thread != current_thread
        && AttachThreadInput(current_thread, target_thread, 1) != 0;
    let attached_foreground = foreground_thread != 0
        && foreground_thread != current_thread
        && foreground_thread != target_thread
        && AttachThreadInput(current_thread, foreground_thread, 1) != 0;

    ShowWindow(hwnd, SW_RESTORE);
    BringWindowToTop(hwnd);
    SetForegroundWindow(hwnd);
    SetActiveWindow(hwnd);

    if attached_foreground {
        AttachThreadInput(current_thread, foreground_thread, 0);
    }
    if attached_target {
        AttachThreadInput(current_thread, target_thread, 0);
    }
}

unsafe fn wait_for_course_creation_completion(
    automation: &IUIAutomation,
    root: &IUIAutomationElement,
    timeout_ms: u64,
) -> Result<(), String> {
    let mut waited = 0;
    while waited <= timeout_ms {
        if STOP_TYPING.load(Ordering::SeqCst) {
            return Ok(());
        }

        let prompt_hwnd = find_top_level_window("提示信息");
        if prompt_hwnd != 0 && IsWindowVisible(prompt_hwnd) != 0 {
            log_event("检测到“提示信息”窗口，准备确认");
            let dialog = automation
                .ElementFromHandle(WindowsHwnd(prompt_hwnd as *mut std::ffi::c_void))
                .map_err(|error| format!("读取“提示信息”窗口失败：{}", error))?;
            confirm_dialog_with_fallback(automation, root, &dialog, "提示信息")?;
        }

        let allergy_hwnd = find_top_level_window("过敏史");
        if allergy_hwnd != 0 && IsWindowVisible(allergy_hwnd) != 0 {
            log_event("过敏史窗口仍存在；立即发送Enter再次确认");
            force_foreground_window(allergy_hwnd);
            press_named_key("Enter");
        }

        let template_hwnd = find_top_level_window("模板列表");
        if template_hwnd == 0 || IsWindowVisible(template_hwnd) == 0 {
            return Ok(());
        }
        if waited % 1000 == 0 {
            log_event(&format!(
                "等待模板列表关闭：已等待{}ms，template_hwnd=0x{:X}",
                waited, template_hwnd
            ));
        }
        thread::sleep(Duration::from_millis(70));
        waited += 70;
    }
    log_event("等待模板列表关闭超时");
    Err("模板确认后窗口未关闭".to_string())
}

unsafe fn find_emr_window(
    automation: &IUIAutomation,
    root: &IUIAutomationElement,
) -> Result<IUIAutomationElement, String> {
    let foreground = GetForegroundWindow();
    if foreground != 0 {
        if let Ok(element) =
            automation.ElementFromHandle(WindowsHwnd(foreground as *mut std::ffi::c_void))
        {
            let name = element
                .CurrentName()
                .map(|value| value.to_string())
                .unwrap_or_default();
            if is_emr_window_title(&name) {
                return Ok(element);
            }
        }
    }

    let condition = automation
        .CreateTrueCondition()
        .map_err(|error| format!("创建窗口查找条件失败：{}", error))?;
    let children = root
        .FindAll(TreeScope_Children, &condition)
        .map_err(|error| format!("枚举桌面窗口失败：{}", error))?;
    let count = children.Length().unwrap_or(0);
    for index in 0..count {
        if let Ok(element) = children.GetElement(index) {
            if element.CurrentControlType().ok() == Some(UIA_WindowControlTypeId) {
                let name = element
                    .CurrentName()
                    .map(|value| value.to_string())
                    .unwrap_or_default();
                if is_emr_window_title(&name) {
                    return Ok(element);
                }
            }
        }
    }
    Err("未找到当前打开的嘉和/嘉禾电子病历平台窗口".to_string())
}

fn is_emr_window_title(title: &str) -> bool {
    let normalized: String = title
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .flat_map(|ch| ch.to_lowercase())
        .collect();
    let has_product_name = normalized.contains("电子病历平台")
        || normalized.contains("嘉和电子病历")
        || normalized.contains("嘉禾电子病历");
    let has_vendor_or_version =
        normalized.contains("嘉和") || normalized.contains("嘉禾") || normalized.contains("v6");
    has_product_name && has_vendor_or_version
}

unsafe fn named_control_condition(
    automation: &IUIAutomation,
    name: &str,
    control_type: i32,
) -> Result<windows::Win32::UI::Accessibility::IUIAutomationCondition, String> {
    let name_value = VARIANT::from(BSTR::from(name));
    let name_condition = automation
        .CreatePropertyCondition(UIA_NamePropertyId, &name_value)
        .map_err(|error| format!("创建名称条件失败：{}", error))?;
    let type_value = VARIANT::from(control_type);
    let type_condition = automation
        .CreatePropertyCondition(UIA_ControlTypePropertyId, &type_value)
        .map_err(|error| format!("创建控件类型条件失败：{}", error))?;
    automation
        .CreateAndCondition(&name_condition, &type_condition)
        .map_err(|error| format!("组合控件条件失败：{}", error))
}

unsafe fn find_last_named_control(
    automation: &IUIAutomation,
    scope: &IUIAutomationElement,
    name: &str,
    control_type: i32,
) -> Result<IUIAutomationElement, String> {
    let condition = named_control_condition(automation, name, control_type)?;
    let elements = scope
        .FindAll(TreeScope_Descendants, &condition)
        .map_err(|error| format!("查找“{}”失败：{}", name, error))?;
    let count = elements.Length().unwrap_or(0);
    if count <= 0 {
        return Err(format!("未找到“{}”", name));
    }
    elements
        .GetElement(count - 1)
        .map_err(|error| format!("读取“{}”失败：{}", name, error))
}

unsafe fn wait_for_template_window(
    automation: &IUIAutomation,
    timeout_ms: u64,
) -> Result<IUIAutomationElement, String> {
    let mut waited = 0;
    while waited <= timeout_ms {
        if STOP_TYPING.load(Ordering::SeqCst) {
            return Err("用户已停止自动化".to_string());
        }
        let hwnd = find_top_level_window("模板列表");
        if hwnd != 0 && IsWindowVisible(hwnd) != 0 {
            return automation
                .ElementFromHandle(WindowsHwnd(hwnd as *mut std::ffi::c_void))
                .map_err(|error| format!("读取“模板列表”窗口失败：{}", error));
        }
        thread::sleep(Duration::from_millis(50));
        waited += 50;
    }
    Err("等待“模板列表”超时".to_string())
}

unsafe fn wait_for_named_control(
    automation: &IUIAutomation,
    scope: &IUIAutomationElement,
    name: &str,
    control_type: i32,
    timeout_ms: u64,
) -> Result<IUIAutomationElement, String> {
    let condition = named_control_condition(automation, name, control_type)?;
    let mut waited = 0;
    while waited <= timeout_ms {
        if STOP_TYPING.load(Ordering::SeqCst) {
            return Err("用户已停止自动化".to_string());
        }
        if let Ok(element) = scope.FindFirst(TreeScope_Descendants, &condition) {
            return Ok(element);
        }
        thread::sleep(Duration::from_millis(20));
        waited += 20;
    }
    Err(format!("等待“{}”超时", name))
}

unsafe fn find_first_control_by_type(
    automation: &IUIAutomation,
    scope: &IUIAutomationElement,
    control_type: i32,
) -> Result<IUIAutomationElement, String> {
    let type_value = VARIANT::from(control_type);
    let condition = automation
        .CreatePropertyCondition(UIA_ControlTypePropertyId, &type_value)
        .map_err(|error| format!("创建控件类型条件失败：{}", error))?;
    scope
        .FindFirst(TreeScope_Descendants, &condition)
        .map_err(|error| format!("查找控件失败：{}", error))
}

unsafe fn select_tree_item(element: &IUIAutomationElement, name: &str) -> Result<(), String> {
    if let Ok(scroll) =
        element.GetCurrentPatternAs::<IUIAutomationScrollItemPattern>(UIA_ScrollItemPatternId)
    {
        let _ = scroll.ScrollIntoView();
    }
    move_cursor_to_element(element);
    let selection: IUIAutomationSelectionItemPattern = element
        .GetCurrentPatternAs(UIA_SelectionItemPatternId)
        .map_err(|_| format!("“{}”不支持SelectionItemPattern", name))?;
    selection
        .Select()
        .map_err(|error| format!("选中“{}”失败：{}", name, error))?;
    element
        .SetFocus()
        .map_err(|error| format!("聚焦“{}”失败：{}", name, error))
}

unsafe fn click_named_button_nonblocking(
    automation: &IUIAutomation,
    scope: &IUIAutomationElement,
    name: &str,
) -> Result<(), String> {
    let button = find_last_named_control(automation, scope, name, UIA_ButtonControlTypeId.0)?;
    let rect = button
        .CurrentBoundingRectangle()
        .map_err(|error| format!("读取按钮“{}”位置失败：{}", name, error))?;
    if rect.right <= rect.left || rect.bottom <= rect.top {
        return Err(format!("按钮“{}”没有有效的屏幕位置", name));
    }
    let x = rect.left + (rect.right - rect.left) / 2;
    let y = rect.top + (rect.bottom - rect.top) / 2;
    SetCursorPos(x, y);
    course_sleep(140);
    mouse_left_click();
    Ok(())
}

unsafe fn move_cursor_to_element(element: &IUIAutomationElement) {
    if let Ok(rect) = element.CurrentBoundingRectangle() {
        if rect.right > rect.left && rect.bottom > rect.top {
            SetCursorPos(
                rect.left + (rect.right - rect.left) / 2,
                rect.top + (rect.bottom - rect.top) / 2,
            );
        }
    }
}

unsafe fn set_template_document_title(
    automation: &IUIAutomation,
    template_window: &IUIAutomationElement,
    title: &str,
) -> Result<(), String> {
    let edits = find_controls_by_type(automation, template_window, UIA_EditControlTypeId.0)?;
    let candidates: Vec<(Rect, IUIAutomationElement)> = edits
        .into_iter()
        .filter_map(|element| {
            let rect = element.CurrentBoundingRectangle().ok()?;
            (rect.right > rect.left && rect.bottom > rect.top).then_some((
                Rect {
                    left: rect.left,
                    top: rect.top,
                    right: rect.right,
                    bottom: rect.bottom,
                },
                element,
            ))
        })
        .collect();
    let candidate_rects: Vec<Rect> = candidates.iter().map(|(rect, _)| *rect).collect();
    let title_edit = document_title_candidate_index(&candidate_rects)
        .and_then(|index| candidates.get(index))
        .map(|(_, element)| element.clone())
        .ok_or_else(|| "未找到底部文档标题输入框".to_string())?;

    move_cursor_to_element(&title_edit);
    title_edit
        .SetFocus()
        .map_err(|error| format!("聚焦文档标题输入框失败：{}", error))?;
    course_sleep(80);
    press_ctrl_a();
    course_sleep(60);
    press_named_key("Delete");
    course_sleep(60);
    type_literal_text(title);
    course_sleep(120);

    if element_readable_values(&title_edit)
        .iter()
        .any(|value| value.trim() == title)
    {
        return Ok(());
    }

    if let Ok(value) =
        title_edit.GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId)
    {
        value
            .SetValue(&BSTR::from(title))
            .map_err(|error| format!("写入文档标题失败：{}", error))?;
        course_sleep(120);
        if element_readable_values(&title_edit)
            .iter()
            .any(|current| current.trim() == title)
        {
            return Ok(());
        }
    }

    Err(format!("文档标题写入后校验失败，期望：{}", title))
}

fn document_title_candidate_index(rects: &[Rect]) -> Option<usize> {
    let bottom_top = rects.iter().map(|rect| rect.top).max()?;
    let bottom_row_threshold = bottom_top.saturating_sub(12);
    rects
        .iter()
        .enumerate()
        .filter(|(_, rect)| rect.top >= bottom_row_threshold)
        .min_by_key(|(_, rect)| rect.left)
        .map(|(index, _)| index)
}

unsafe fn find_template_item(
    automation: &IUIAutomation,
    template_window: &IUIAutomationElement,
    template_name: &str,
) -> Result<IUIAutomationElement, String> {
    if let Ok(element) = find_last_named_control(
        automation,
        template_window,
        template_name,
        UIA_TreeItemControlTypeId.0,
    ) {
        return Ok(element);
    }

    let edits = find_controls_by_type(automation, template_window, UIA_EditControlTypeId.0)?;
    let search = edits
        .into_iter()
        .min_by_key(|element| {
            element
                .CurrentBoundingRectangle()
                .map(|rect| rect.top)
                .unwrap_or(i32::MAX)
        })
        .ok_or_else(|| "未找到模板检索框".to_string())?;
    let value: IUIAutomationValuePattern = search
        .GetCurrentPatternAs(UIA_ValuePatternId)
        .map_err(|_| "模板检索框不支持ValuePattern".to_string())?;
    let search_text = BSTR::from(template_name);
    value
        .SetValue(&search_text)
        .map_err(|error| format!("写入模板检索词失败：{}", error))?;
    search
        .SetFocus()
        .map_err(|error| format!("聚焦模板检索框失败：{}", error))?;
    press_named_key("Enter");
    course_sleep(260);
    find_last_named_control(
        automation,
        template_window,
        template_name,
        UIA_TreeItemControlTypeId.0,
    )
    .map_err(|_| format!("模板列表中未找到“{}”", template_name))
}

unsafe fn find_controls_by_type(
    automation: &IUIAutomation,
    scope: &IUIAutomationElement,
    control_type: i32,
) -> Result<Vec<IUIAutomationElement>, String> {
    let type_value = VARIANT::from(control_type);
    let condition = automation
        .CreatePropertyCondition(UIA_ControlTypePropertyId, &type_value)
        .map_err(|error| format!("创建控件类型条件失败：{}", error))?;
    let array = scope
        .FindAll(TreeScope_Descendants, &condition)
        .map_err(|error| format!("枚举控件失败：{}", error))?;
    let mut result = Vec::new();
    for index in 0..array.Length().unwrap_or(0) {
        if let Ok(element) = array.GetElement(index) {
            result.push(element);
        }
    }
    Ok(result)
}

unsafe fn set_template_datetime(
    automation: &IUIAutomation,
    template_window: &IUIAutomationElement,
    datetime: &str,
    date: SimpleDate,
    clock: &str,
) -> Result<(), String> {
    let date_control = find_template_datetime_control(automation, template_window)?;
    let exposes_full_datetime = element_readable_values(&date_control)
        .iter()
        .any(|value| parse_datetime_parts(value).is_some());

    if exposes_full_datetime {
        if let Ok(value) =
            date_control.GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId)
        {
            let target = BSTR::from(datetime);
            if value.SetValue(&target).is_ok() {
                course_sleep(180);
                if element_datetime_matches(&date_control, datetime) {
                    return Ok(());
                }
            }
        }

        if let Ok(legacy) = date_control
            .GetCurrentPatternAs::<IUIAutomationLegacyIAccessiblePattern>(
                UIA_LegacyIAccessiblePatternId,
            )
        {
            let target = wide(datetime);
            if legacy.SetValue(PCWSTR(target.as_ptr())).is_ok() {
                course_sleep(180);
                if element_datetime_matches(&date_control, datetime) {
                    return Ok(());
                }
            }
        }
    }

    date_control
        .SetFocus()
        .map_err(|error| format!("聚焦模板时间失败：{}", error))?;
    course_sleep(120);
    for _ in 0..10 {
        press_named_key("Left");
        course_sleep(35);
    }
    let clock_parts: Vec<&str> = clock.split(':').collect();
    let parts = [
        format!("{:04}", date.year),
        format!("{:02}", date.month),
        format!("{:02}", date.day),
        clock_parts.get(0).copied().unwrap_or("08").to_string(),
        clock_parts.get(1).copied().unwrap_or("00").to_string(),
        clock_parts.get(2).copied().unwrap_or("00").to_string(),
    ];
    for (index, part) in parts.iter().enumerate() {
        type_literal_text(part);
        course_sleep(90);
        if index + 1 < parts.len() {
            press_named_key("Right");
            course_sleep(90);
        }
    }
    course_sleep(180);

    let readable = element_readable_values(&date_control);
    if readable
        .iter()
        .any(|value| parse_datetime_parts(value).is_some())
    {
        if !element_datetime_matches(&date_control, datetime) {
            return Err(format!(
                "时间写入后校验失败，控件当前值：{}",
                readable.join(" / ")
            ));
        }
    } else if readable
        .iter()
        .any(|value| parse_date_parts(value).is_some())
    {
        let expected_date = [date.year, date.month as i32, date.day as i32];
        let date_matches = readable
            .iter()
            .filter_map(|value| parse_date_parts(value))
            .any(|parts| parts == expected_date);
        if !date_matches {
            return Err(format!(
                "日期写入后校验失败，控件当前值：{}",
                readable.join(" / ")
            ));
        }
    }
    Ok(())
}

unsafe fn find_template_datetime_control(
    automation: &IUIAutomation,
    template_window: &IUIAutomationElement,
) -> Result<IUIAutomationElement, String> {
    let condition = automation
        .CreateTrueCondition()
        .map_err(|error| format!("创建时间控件查找条件失败：{}", error))?;
    let elements = template_window
        .FindAll(TreeScope_Descendants, &condition)
        .map_err(|error| format!("枚举模板时间控件失败：{}", error))?;
    let mut selected: Option<(i32, i32, IUIAutomationElement)> = None;

    for index in 0..elements.Length().unwrap_or(0) {
        let Ok(element) = elements.GetElement(index) else {
            continue;
        };
        let values = element_readable_values(&element);
        let has_full_datetime = values
            .iter()
            .any(|value| parse_datetime_parts(value).is_some());
        let has_date = values.iter().any(|value| parse_date_parts(value).is_some());
        if !has_full_datetime && !has_date {
            continue;
        }

        let mut score = 0;
        if has_full_datetime {
            score += 30;
        } else {
            score += 15;
        }
        if element
            .GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId)
            .is_ok()
        {
            score += 20;
        }
        if element
            .GetCurrentPatternAs::<IUIAutomationLegacyIAccessiblePattern>(
                UIA_LegacyIAccessiblePatternId,
            )
            .is_ok()
        {
            score += 10;
        }
        if element
            .CurrentIsKeyboardFocusable()
            .map(|value| value.as_bool())
            .unwrap_or(false)
        {
            score += 20;
        }
        if element.CurrentControlType().ok() == Some(UIA_PaneControlTypeId) {
            score += 10;
        }
        let top = element
            .CurrentBoundingRectangle()
            .map(|rect| rect.top)
            .unwrap_or(0);
        let replace = selected
            .as_ref()
            .map(|(current_score, current_top, _)| {
                score > *current_score || (score == *current_score && top > *current_top)
            })
            .unwrap_or(true);
        if replace {
            selected = Some((score, top, element));
        }
    }

    selected
        .map(|(_, _, element)| element)
        .ok_or_else(|| "未找到包含日期的底部时间控件".to_string())
}

unsafe fn element_readable_values(element: &IUIAutomationElement) -> Vec<String> {
    let mut values = Vec::new();
    if let Ok(name) = element.CurrentName() {
        let name = name.to_string();
        if !name.trim().is_empty() {
            values.push(name);
        }
    }
    if let Ok(value) = element.GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId)
    {
        if let Ok(current) = value.CurrentValue() {
            let current = current.to_string();
            if !current.trim().is_empty() && !values.contains(&current) {
                values.push(current);
            }
        }
    }
    if let Ok(legacy) = element.GetCurrentPatternAs::<IUIAutomationLegacyIAccessiblePattern>(
        UIA_LegacyIAccessiblePatternId,
    ) {
        if let Ok(current) = legacy.CurrentValue() {
            let current = current.to_string();
            if !current.trim().is_empty() && !values.contains(&current) {
                values.push(current);
            }
        }
    }
    values
}

unsafe fn element_datetime_matches(element: &IUIAutomationElement, expected: &str) -> bool {
    let Some(expected_parts) = parse_datetime_parts(expected) else {
        return false;
    };
    element_readable_values(element)
        .iter()
        .filter_map(|value| parse_datetime_parts(value))
        .any(|parts| parts == expected_parts)
}

fn parse_datetime_parts(value: &str) -> Option<[i32; 6]> {
    let numbers: Vec<i32> = value
        .split(|ch: char| !ch.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<i32>().ok())
        .collect();
    for parts in numbers.windows(6) {
        if (2000..=2100).contains(&parts[0])
            && (1..=12).contains(&parts[1])
            && (1..=31).contains(&parts[2])
            && (0..=23).contains(&parts[3])
            && (0..=59).contains(&parts[4])
            && (0..=59).contains(&parts[5])
        {
            return Some([parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]]);
        }
    }
    None
}

fn parse_date_parts(value: &str) -> Option<[i32; 3]> {
    let numbers: Vec<i32> = value
        .split(|ch: char| !ch.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<i32>().ok())
        .collect();
    for parts in numbers.windows(3) {
        if (2000..=2100).contains(&parts[0])
            && (1..=12).contains(&parts[1])
            && (1..=31).contains(&parts[2])
        {
            return Some([parts[0], parts[1], parts[2]]);
        }
    }
    None
}

unsafe fn set_superior(
    automation: &IUIAutomation,
    _root: &IUIAutomationElement,
    template_window: &IUIAutomationElement,
    superior: &str,
) -> Result<(), String> {
    log_event(&format!("上级医师“{}”：准备打开选择框", superior));
    invoke_superior_open_button(automation, template_window)?;
    let popup = wait_for_named_control(
        automation,
        template_window,
        "popupContainerControlSample",
        UIA_PaneControlTypeId.0,
        600,
    )?;
    log_event(&format!("上级医师“{}”：选择框已打开", superior));
    let search_edit = find_first_control_by_type(automation, &popup, UIA_EditControlTypeId.0)?;
    move_cursor_to_element(&search_edit);
    search_edit
        .SetFocus()
        .map_err(|error| format!("聚焦上级医师搜索框失败：{}", error))?;
    mouse_left_click();
    press_ctrl_a();
    type_literal_text(superior);
    log_event(&format!("上级医师“{}”：姓名已输入", superior));

    let item = wait_for_superior_name_cell(automation, &popup, superior, 600)?;
    activate_data_item_element(&item)?;
    log_event(&format!("上级医师“{}”：搜索结果已确认", superior));
    course_sleep(80);
    Ok(())
}

unsafe fn invoke_superior_open_button(
    automation: &IUIAutomation,
    template_window: &IUIAutomationElement,
) -> Result<(), String> {
    let condition = named_control_condition(automation, "Open", UIA_ButtonControlTypeId.0)?;
    let array = template_window
        .FindAll(TreeScope_Descendants, &condition)
        .map_err(|error| format!("查找下拉按钮失败：{}", error))?;
    let count = array.Length().unwrap_or(0);
    if count <= 0 {
        return Err("未找到下拉按钮Open".to_string());
    }
    let mut selected: Option<(i32, IUIAutomationElement)> = None;
    for index in 0..count {
        if let Ok(element) = array.GetElement(index) {
            let left = element
                .CurrentBoundingRectangle()
                .map(|rect| rect.left)
                .unwrap_or(i32::MAX);
            let should_replace = selected
                .as_ref()
                .map(|(current, _)| left < *current)
                .unwrap_or(true);
            if should_replace {
                selected = Some((left, element));
            }
        }
    }
    let button = selected
        .map(|(_, element)| element)
        .ok_or_else(|| "无法读取下拉按钮".to_string())?;
    let invoke: IUIAutomationInvokePattern = button
        .GetCurrentPatternAs(UIA_InvokePatternId)
        .map_err(|_| "下拉按钮不支持InvokePattern".to_string())?;
    invoke
        .Invoke()
        .map_err(|error| format!("打开下拉列表失败：{}", error))
}

unsafe fn wait_for_data_item_value(
    automation: &IUIAutomation,
    popup: &IUIAutomationElement,
    expected: &str,
    timeout_ms: u64,
) -> Result<IUIAutomationElement, String> {
    let mut waited = 0;
    while waited <= timeout_ms {
        if STOP_TYPING.load(Ordering::SeqCst) {
            return Err("用户已停止自动化".to_string());
        }
        let items = find_controls_by_type(automation, popup, UIA_DataItemControlTypeId.0)?;
        for item in items {
            let values = element_readable_values(&item);
            if values
                .iter()
                .any(|value| value.trim() == expected || value.contains(expected))
            {
                return Ok(item);
            }
        }
        let interval = scaled_course_delay(80);
        thread::sleep(Duration::from_millis(interval));
        waited += interval;
    }
    Err(format!(
        "输入上级医师“{}”后，结果表中未找到对应姓名",
        expected
    ))
}

unsafe fn wait_for_superior_name_cell(
    automation: &IUIAutomation,
    popup: &IUIAutomationElement,
    expected: &str,
    timeout_ms: u64,
) -> Result<IUIAutomationElement, String> {
    let condition = named_control_condition(automation, "名称 行 0", UIA_DataItemControlTypeId.0)?;
    let mut waited = 0;
    while waited <= timeout_ms {
        if STOP_TYPING.load(Ordering::SeqCst) {
            return Err("用户已停止自动化".to_string());
        }
        if let Ok(item) = popup.FindFirst(TreeScope_Descendants, &condition) {
            if element_readable_values(&item)
                .iter()
                .any(|value| value.trim() == expected || value.contains(expected))
            {
                return Ok(item);
            }
        }
        thread::sleep(Duration::from_millis(20));
        waited += 20;
    }
    wait_for_data_item_value(automation, popup, expected, 300)
}

unsafe fn activate_data_item_element(item: &IUIAutomationElement) -> Result<(), String> {
    item.SetFocus()
        .map_err(|error| format!("聚焦上级医师结果行失败：{}", error))?;
    if let Ok(legacy) = item.GetCurrentPatternAs::<IUIAutomationLegacyIAccessiblePattern>(
        UIA_LegacyIAccessiblePatternId,
    ) {
        let _ = legacy.DoDefaultAction();
    }
    course_sleep(60);
    press_named_key("Enter");
    Ok(())
}

fn load_create_courses_config() -> CreateCoursesConfig {
    let mut config = CreateCoursesConfig {
        attending_superior: "裴傲".to_string(),
        chief_superior: "裴傲".to_string(),
    };
    if let Ok(text) = fs::read_to_string(create_courses_config_path()) {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "主治上级" => config.attending_superior = value.trim().to_string(),
                    "主任上级" => config.chief_superior = value.trim().to_string(),
                    _ => {}
                }
            }
        }
    }
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn course_schedule_matches_requested_offsets_and_times() {
        let preop_schedule: Vec<(u32, &str)> = PREOP_COURSE_SPECS
            .iter()
            .map(|spec| (spec.day_offset, spec.clock))
            .collect();
        assert_eq!(
            preop_schedule,
            vec![
                (0, "21:00:00"),
                (0, "21:00:00"),
                (1, "08:00:00"),
                (2, "08:00:00"),
                (3, "08:00:00"),
                (1, "08:00:00"),
                (1, "08:00:00"),
                (5, "08:00:00"),
                (5, "08:00:00"),
            ]
        );
        let postop_offsets: Vec<u32> = POSTOP_COURSE_SPECS
            .iter()
            .map(|spec| spec.day_offset)
            .collect();
        let postop_clocks: Vec<&str> = POSTOP_COURSE_SPECS.iter().map(|spec| spec.clock).collect();
        assert_eq!(postop_offsets, vec![0, 1, 2, 3]);
        assert_eq!(
            postop_clocks,
            vec!["20:00:00", "08:00:00", "08:00:00", "08:00:00"]
        );
        assert_eq!(PREOP_COURSE_SPECS.len(), 9);
        assert_eq!(POSTOP_COURSE_SPECS.len(), 4);
        assert_eq!(PREOP_COURSE_SPECS[0].template, "神经外科入院记录二");
        assert_eq!(discharge_certificate_diagnosis_marker(), " ");
        assert!(matches!(
            PREOP_COURSE_SPECS[2].superior_kind,
            Some(SuperiorKind::Attending)
        ));
        assert!(matches!(
            PREOP_COURSE_SPECS[3].superior_kind,
            Some(SuperiorKind::Chief)
        ));
        assert_eq!(PREOP_COURSE_SPECS[5].parent, "讨论记录");
        assert_eq!(PREOP_COURSE_SPECS[6].parent, "讨论记录");
    }

    #[test]
    fn dock_hot_zone_is_limited_to_the_docked_window_span() {
        let window = Rect {
            left: 1906,
            top: 180,
            right: 2218,
            bottom: 1070,
        };
        assert!(cursor_in_dock_hot_zone(
            DOCK_EDGE_RIGHT,
            Point { x: 1919, y: 620 },
            window,
            1920
        ));
        assert!(!cursor_in_dock_hot_zone(
            DOCK_EDGE_RIGHT,
            Point { x: 1919, y: 200 },
            window,
            1920
        ));
        assert!(!cursor_in_dock_hot_zone(
            DOCK_EDGE_RIGHT,
            Point { x: 1919, y: 1000 },
            window,
            1920
        ));

        let top_window = Rect {
            left: 420,
            top: -876,
            right: 746,
            bottom: 14,
        };
        assert!(cursor_in_dock_hot_zone(
            DOCK_EDGE_TOP,
            Point { x: 500, y: 2 },
            top_window,
            1920
        ));
        assert!(!cursor_in_dock_hot_zone(
            DOCK_EDGE_TOP,
            Point { x: 40, y: 2 },
            top_window,
            1920
        ));
    }

    #[test]
    fn empty_launch_path_uses_default_value() {
        assert_eq!(
            setting_or_default("  ", default_medical_system_path()),
            default_medical_system_path()
        );
        assert_eq!(setting_or_default("D:\\app.exe", "fallback"), "D:\\app.exe");
    }

    #[test]
    fn postop_titles_use_chief_and_split_day_two_and_three() {
        let config = CreateCoursesConfig {
            attending_superior: "张东".to_string(),
            chief_superior: "裴傲".to_string(),
        };
        assert_eq!(
            resolve_course_document_title(POSTOP_COURSE_SPECS[1], &config).as_deref(),
            Some("术后第1日裴傲术者查房记录")
        );
        assert_eq!(
            resolve_course_document_title(POSTOP_COURSE_SPECS[2], &config).as_deref(),
            Some("术后第2日查房记录")
        );
        assert_eq!(
            resolve_course_document_title(POSTOP_COURSE_SPECS[3], &config).as_deref(),
            Some("术后第3日查房记录")
        );
        for spec in &POSTOP_COURSE_SPECS {
            assert!(spec.superior_kind.is_none());
        }
    }

    #[test]
    fn document_title_uses_leftmost_edit_on_bottom_row() {
        let rects = [
            Rect {
                left: 10,
                top: 80,
                right: 190,
                bottom: 104,
            },
            Rect {
                left: 120,
                top: 500,
                right: 340,
                bottom: 524,
            },
            Rect {
                left: 540,
                top: 501,
                right: 670,
                bottom: 525,
            },
            Rect {
                left: 710,
                top: 501,
                right: 810,
                bottom: 525,
            },
        ];
        assert_eq!(document_title_candidate_index(&rects), Some(1));
    }

    #[test]
    fn add_days_handles_month_boundary() {
        let result = add_days(
            SimpleDate {
                year: 2026,
                month: 7,
                day: 30,
            },
            5,
        );
        assert_eq!((result.year, result.month, result.day), (2026, 8, 4));
    }

    #[test]
    fn emr_window_title_match_is_not_tied_to_login_name() {
        assert!(is_emr_window_title("裴傲-嘉和电子病历平台V6.0"));
        assert!(is_emr_window_title("卢盛华  -  嘉禾电子病历平台 V6.0.3"));
        assert!(is_emr_window_title("神经外科电子病历平台 v6"));
        assert!(!is_emr_window_title("模板列表"));
        assert!(!is_emr_window_title("神外小助手"));
    }

    #[test]
    fn datetime_control_values_are_parsed_for_verification() {
        assert_eq!(
            parse_datetime_parts("2026-07-17 18:28:41"),
            Some([2026, 7, 17, 18, 28, 41])
        );
        assert_eq!(
            parse_datetime_parts("时间：2026年7月18日 8时00分00秒"),
            Some([2026, 7, 18, 8, 0, 0])
        );
        assert_eq!(parse_datetime_parts("2026年7月18日"), None);
        assert_eq!(parse_date_parts("2026年7月17日"), Some([2026, 7, 17]));
    }

    #[test]
    fn clinical_path_2160_script_preserves_recorded_actions() {
        let script = String::from_utf8_lossy(include_bytes!("../scripts/临床路径2160p.Q"));
        assert_eq!(
            script
                .lines()
                .filter(|line| line.starts_with("LeftDown"))
                .count(),
            48
        );
        assert_eq!(
            script
                .lines()
                .filter(|line| line.starts_with("LeftUp"))
                .count(),
            48
        );
        assert_eq!(
            script
                .lines()
                .filter(|line| line.starts_with("KeyDown"))
                .count(),
            14
        );
        assert_eq!(
            script
                .lines()
                .filter(|line| line.starts_with("KeyUp"))
                .count(),
            14
        );
        assert!(script.contains("MoveTo 1144, 1006"));
        assert!(script.contains("MoveTo 1593, 455"));
    }

    #[test]
    fn superior_popup_stays_on_screen_and_opens_up_when_needed() {
        let anchor = Rect {
            left: 1800,
            top: 780,
            right: 1910,
            bottom: 812,
        };
        assert_eq!(
            selection_popup_origin(anchor, 1920, 1080, 270, 326),
            (1640, 450)
        );

        let upper_anchor = Rect {
            left: 100,
            top: 100,
            right: 232,
            bottom: 132,
        };
        assert_eq!(
            selection_popup_origin(upper_anchor, 1920, 1080, 270, 326),
            (100, 136)
        );
    }
}

fn press_alt_combo(vk: u16) {
    unsafe {
        send_virtual_key(0x12, false);
        sleep_interruptible(60);
        send_virtual_key(vk, false);
        sleep_interruptible(60);
        send_virtual_key(vk, true);
        sleep_interruptible(60);
        send_virtual_key(0x12, true);
    }
}

fn press_ctrl_a() {
    unsafe {
        send_virtual_key(0x11, false);
        sleep_interruptible(60);
        send_virtual_key(0x41, false);
        sleep_interruptible(60);
        send_virtual_key(0x41, true);
        sleep_interruptible(60);
        send_virtual_key(0x11, true);
    }
}

unsafe fn start_clinical_path_flow() {
    if CLINICAL_PATH_RUNNING.swap(true, Ordering::SeqCst) {
        set_status("临床路径正在执行。");
        return;
    }
    STOP_TYPING.store(false, Ordering::SeqCst);
    set_status("正在执行临床路径...");
    thread::spawn(|| {
        run_clinical_path_flow();
        CLINICAL_PATH_RUNNING.store(false, Ordering::SeqCst);
    });
}

unsafe fn start_clinical_path_continuous_flow() {
    if CLINICAL_PATH_RUNNING.swap(true, Ordering::SeqCst) {
        set_status("临床路径正在执行。");
        return;
    }
    let loop_limit = clinical_loop_count_limit();
    save_settings(&read_settings_from_ui());
    STOP_TYPING.store(false, Ordering::SeqCst);
    if let Some(limit) = loop_limit {
        set_status(&format!(
            "正在持续执行临床路径，共 {} 次；右Ctrl+右Alt可终止。",
            limit
        ));
    } else {
        set_status("正在持续执行临床路径；右Ctrl+右Alt终止。");
    }
    thread::spawn(move || {
        let mut completed = 0u32;
        while !STOP_TYPING.load(Ordering::SeqCst) {
            run_clinical_path_flow();
            completed += 1;
            if STOP_TYPING.load(Ordering::SeqCst) {
                break;
            }
            if loop_limit.map(|limit| completed >= limit).unwrap_or(false) {
                break;
            }
            clinical_path_next_item();
        }
        CLINICAL_PATH_RUNNING.store(false, Ordering::SeqCst);
    });
}

fn run_clinical_path_flow() {
    let profile = CLINICAL_PATH_RESOLUTION_PROFILE
        .load(Ordering::SeqCst)
        .min(1);
    if profile == 0 {
        mouse_move(730, 820);
        sleep_interruptible(450);
        mouse_move(815, 826);
        sleep_interruptible(450);
    }
    let script_name = if profile == 1 {
        "\u{4e34}\u{5e8a}\u{8def}\u{5f84}2160p.Q"
    } else {
        "\u{4e34}\u{5e8a}\u{8def}\u{5f84}.Q"
    };
    let text = fs::read_to_string(script_path(script_name))
        .or_else(|_| fs::read_to_string(asset_path("scripts", script_name)))
        .or_else(|_| match profile {
            0 => fs::read_to_string(r"C:\Users\卢盛华\Desktop\DeskShare\互通\vibeC\临床路径.Q"),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no legacy 2160p script",
            )),
        })
        .or_else(|_| {
            fs::read_to_string(
                Path::new(r"C:\Users\卢盛华\Desktop\DeskShare\互通\resident_typer\scripts")
                    .join(script_name),
            )
        });
    let script = text.unwrap_or_else(|_| match profile {
        1 => String::from_utf8_lossy(include_bytes!("../scripts/临床路径2160p.Q")).into_owned(),
        _ => String::from_utf8_lossy(include_bytes!("../scripts/临床路径.Q")).into_owned(),
    });
    if profile == 0 {
        mouse_move(730, 820);
        sleep_interruptible(450);
    }
    run_q_mouse_keyboard_script(&script, profile == 0);
}

fn clinical_path_next_item() {
    press_named_key("Tab");
    sleep_interruptible(180);
    press_named_key("Tab");
    sleep_interruptible(180);
    press_named_key("Down");
    sleep_interruptible(180);
    press_named_key("Down");
    sleep_interruptible(260);
}

fn run_q_mouse_keyboard_script(script: &str, slow: bool) {
    for raw in script.lines() {
        if STOP_TYPING.load(Ordering::SeqCst) {
            return;
        }
        let line = raw.trim();
        if line.is_empty() || line.starts_with('\'') || line.starts_with('[') {
            continue;
        }
        if let Some(ms) = parse_delay(line) {
            sleep_interruptible(scaled_script_delay(ms, slow));
        } else if let Some((x, y)) = parse_move_to(line) {
            mouse_move(x, y);
            if slow {
                sleep_interruptible(100);
            }
        } else if line.starts_with("LeftClick") {
            mouse_left_click();
            if slow {
                sleep_interruptible(350);
            }
        } else if line.starts_with("LeftDown") {
            mouse_left_down();
            if slow {
                sleep_interruptible(140);
            }
        } else if line.starts_with("LeftUp") {
            mouse_left_up();
            if slow {
                sleep_interruptible(200);
            }
        } else if let Some((key, count)) = parse_key_command(line, "KeyPress") {
            for _ in 0..count {
                press_macro_key(&key);
                if slow {
                    sleep_interruptible(140);
                }
            }
        } else if let Some((key, count)) = parse_key_command(line, "KeyDown") {
            for _ in 0..count {
                if let Some(vk) = macro_key_to_vk(&key) {
                    unsafe { send_virtual_key(vk, false) };
                    if slow {
                        sleep_interruptible(100);
                    }
                }
            }
        } else if let Some((key, count)) = parse_key_command(line, "KeyUp") {
            for _ in 0..count {
                if let Some(vk) = macro_key_to_vk(&key) {
                    unsafe { send_virtual_key(vk, true) };
                    if slow {
                        sleep_interruptible(125);
                    }
                }
            }
        } else if let Some(text) = parse_say_string(line) {
            type_text(&text);
            if slow {
                sleep_interruptible(200);
            }
        } else if let Some(path) = parse_run_app(line) {
            spawn_with_program_directory(&path);
            if slow {
                sleep_interruptible(540);
            }
        }
    }
}

fn scaled_script_delay(ms: u64, slow: bool) -> u64 {
    if slow {
        (((ms as f64) * 0.77).round() as u64).clamp(120, 1800)
    } else {
        ms
    }
}

fn parse_delay(line: &str) -> Option<u64> {
    line.strip_prefix("Delay ")?.trim().parse::<u64>().ok()
}

fn parse_move_to(line: &str) -> Option<(i32, i32)> {
    let rest = line.strip_prefix("MoveTo ")?.trim();
    let (x, y) = rest.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}

fn parse_key_command(line: &str, command: &str) -> Option<(String, usize)> {
    let rest = line.strip_prefix(command)?.trim();
    if let Some(first_quote) = rest.find('"') {
        let after_first = &rest[first_quote + 1..];
        let second_quote = after_first.find('"')?;
        let key = after_first[..second_quote].to_string();
        let count = after_first[second_quote + 1..]
            .split(',')
            .nth(1)
            .and_then(|n| n.trim().parse::<usize>().ok())
            .unwrap_or(1);
        return Some((key, count));
    }
    let (key, count) = rest.split_once(',').unwrap_or((rest, "1"));
    Some((key.trim().to_string(), count.trim().parse().unwrap_or(1)))
}

fn parse_say_string(line: &str) -> Option<String> {
    let rest = line.strip_prefix("SayString ")?.trim();
    let first_quote = rest.find('"')?;
    let after_first = &rest[first_quote + 1..];
    let second_quote = after_first.find('"')?;
    Some(after_first[..second_quote].to_string())
}

fn parse_run_app(line: &str) -> Option<String> {
    let rest = line.strip_prefix("RunApp ")?.trim();
    let first_quote = rest.find('"')?;
    let after_first = &rest[first_quote + 1..];
    let second_quote = after_first.find('"')?;
    Some(after_first[..second_quote].to_string())
}

fn press_macro_key(key: &str) {
    if let Some(vk) = macro_key_to_vk(key) {
        unsafe {
            send_virtual_key(vk, false);
            sleep_interruptible(35);
            send_virtual_key(vk, true);
        }
        sleep_interruptible(60);
    } else {
        type_text(key);
    }
}

fn macro_key_to_vk(key: &str) -> Option<u16> {
    if let Ok(vk) = key.parse::<u16>() {
        return Some(vk);
    }
    key_to_vk(key).or_else(|| {
        if key.len() == 1 {
            let ch = key.chars().next().unwrap();
            if ch.is_ascii_alphanumeric() {
                return Some(ch.to_ascii_uppercase() as u16);
            }
        }
        None
    })
}

fn mouse_move(x: i32, y: i32) {
    unsafe {
        SetCursorPos(x, y);
        send_mouse_input(x, y, MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE);
    }
}

fn mouse_left_down() {
    unsafe {
        send_mouse_button(MOUSEEVENTF_LEFTDOWN);
    }
}

fn mouse_left_up() {
    unsafe {
        send_mouse_button(MOUSEEVENTF_LEFTUP);
    }
}

fn mouse_left_click() {
    mouse_left_down();
    sleep_interruptible(40);
    mouse_left_up();
}

unsafe fn send_mouse_input(x: i32, y: i32, flags: u32) {
    let width = GetSystemMetrics(SM_CXSCREEN).max(1) - 1;
    let height = GetSystemMetrics(SM_CYSCREEN).max(1) - 1;
    let absolute_x = (x.clamp(0, width) * 65535) / width.max(1);
    let absolute_y = (y.clamp(0, height) * 65535) / height.max(1);
    let input = Input {
        input_type: INPUT_MOUSE,
        u: InputUnion {
            mi: MouseInput {
                dx: absolute_x,
                dy: absolute_y,
                mouse_data: 0,
                dw_flags: flags,
                time: 0,
                dw_extra_info: 0,
            },
        },
    };
    SendInput(1, &input, size_of::<Input>() as i32);
}

unsafe fn send_mouse_button(flags: u32) {
    let input = Input {
        input_type: INPUT_MOUSE,
        u: InputUnion {
            mi: MouseInput {
                dx: 0,
                dy: 0,
                mouse_data: 0,
                dw_flags: flags,
                time: 0,
                dw_extra_info: 0,
            },
        },
    };
    SendInput(1, &input, size_of::<Input>() as i32);
}

fn render_template(template: &str, options: Options) -> String {
    let vitals = random_vitals(options);
    let temp = format!("{}.{}", vitals.temp10 / 10, vitals.temp10 % 10);
    template
        .replace("{SBP}", &vitals.sbp.to_string())
        .replace("{DBP}", &vitals.dbp.to_string())
        .replace("{HR}", &vitals.hr.to_string())
        .replace("{RR}", &vitals.rr.to_string())
        .replace("{TEMP}", &temp)
        .replace("{CONDITIONS}", &condition_text(options))
}

fn condition_text(options: Options) -> String {
    let mut parts = Vec::new();
    if options.hypertension {
        parts.push("既往高血压，注意血压控制");
    }
    if options.diabetes {
        parts.push("合并糖尿病，注意血糖监测");
    }
    if options.fast_hr {
        parts.push("心率偏快，需动态观察");
    }
    if options.severe {
        parts.push("病情偏重，需严密监护");
    }
    if parts.is_empty() {
        "未勾选特殊情况".to_string()
    } else {
        parts.join("；")
    }
}

fn random_vitals(options: Options) -> Vitals {
    let mut rng = SimpleRng::new();
    let sbp = if options.hypertension {
        rng.range(150, 160)
    } else {
        rng.range(130, 140)
    };
    let dbp = if options.hypertension {
        rng.range(90, 100)
    } else {
        rng.range(75, 88)
    };
    let hr = if options.fast_hr {
        rng.range(105, 125)
    } else {
        rng.range(76, 96)
    };
    let rr = if options.severe {
        rng.range(22, 28)
    } else {
        rng.range(16, 20)
    };
    let temp10 = if options.severe {
        rng.range(373, 385)
    } else {
        rng.range(363, 371)
    };
    Vitals {
        sbp,
        dbp,
        hr,
        rr,
        temp10,
    }
}

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(88172645463393265);
        Self {
            state: nanos ^ 0x9E3779B97F4A7C15,
        }
    }

    fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }

    fn range(&mut self, min: u32, max: u32) -> u32 {
        min + (self.next() % (max - min + 1))
    }
}

unsafe fn start_clipboard_typing_flow() {
    if CLIPBOARD_TYPING_RUNNING.swap(true, Ordering::SeqCst) {
        set_status("剪贴板内容正在输入。");
        return;
    }
    STOP_TYPING.store(false, Ordering::SeqCst);
    set_status("已读取快捷键；松开 Ctrl 和 Alt 后开始逐字输入。");
    thread::spawn(|| {
        let text = unsafe { read_clipboard_unicode_text() };
        let result = match text {
            Ok(text) if text.is_empty() => Err("剪贴板中没有文字".to_string()),
            Ok(text) => {
                let mut waited = 0;
                while (CTRL_DOWN.load(Ordering::SeqCst) || ALT_DOWN.load(Ordering::SeqCst))
                    && waited < 2000
                {
                    thread::sleep(Duration::from_millis(20));
                    waited += 20;
                }
                // Ctrl+Alt+V is intercepted by the low-level hook. Restore
                // the original input window without replacing its child-edit focus.
                let target = CLIPBOARD_TARGET_WINDOW.load(Ordering::SeqCst);
                if target != 0 && target != APP.hwnd && IsWindow(target) != 0 {
                    restore_clipboard_target_window(target);
                    thread::sleep(Duration::from_millis(120));
                }
                thread::sleep(Duration::from_millis(40));
                type_clipboard_text_fast(&text);
                Ok(())
            }
            Err(error) => Err(error),
        };
        CLIPBOARD_TYPING_RUNNING.store(false, Ordering::SeqCst);
        unsafe {
            if STOP_TYPING.load(Ordering::SeqCst) {
                set_status("剪贴板逐字输入已停止。");
            } else if let Err(error) = result {
                set_status(&format!("剪贴板自动输入失败：{}", error));
            } else {
                set_status("剪贴板内容已逐字输入完成。");
            }
        }
    });
}

unsafe fn read_clipboard_unicode_text() -> Result<String, String> {
    if IsClipboardFormatAvailable(CF_UNICODETEXT) == 0 {
        return Err("剪贴板中没有 Unicode 文本".to_string());
    }
    let mut opened = false;
    for _ in 0..8 {
        if OpenClipboard(0) != 0 {
            opened = true;
            break;
        }
        thread::sleep(Duration::from_millis(25));
    }
    if !opened {
        return Err("剪贴板正被其他程序占用".to_string());
    }

    let handle = GetClipboardData(CF_UNICODETEXT);
    if handle == 0 {
        CloseClipboard();
        return Err("无法读取剪贴板文本".to_string());
    }
    let pointer = GlobalLock(handle);
    if pointer.is_null() {
        CloseClipboard();
        return Err("无法锁定剪贴板文本".to_string());
    }

    let mut length = 0usize;
    const MAX_CLIPBOARD_UNITS: usize = 2_000_000;
    while length < MAX_CLIPBOARD_UNITS && *pointer.add(length) != 0 {
        length += 1;
    }
    let text = String::from_utf16_lossy(std::slice::from_raw_parts(pointer, length));
    GlobalUnlock(handle);
    CloseClipboard();
    Ok(text)
}

fn type_clipboard_text_fast(text: &str) {
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if STOP_TYPING.load(Ordering::SeqCst) {
            return;
        }
        match ch {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                press_named_key("Enter");
                thread::sleep(Duration::from_millis(35));
            }
            '\n' => {
                press_named_key("Enter");
                thread::sleep(Duration::from_millis(35));
            }
            '\t' => {
                press_named_key("Tab");
                thread::sleep(Duration::from_millis(35));
            }
            _ => {
                for unit in ch.encode_utf16(&mut [0; 2]) {
                    unsafe {
                        send_unicode_key(*unit, false);
                        send_unicode_key(*unit, true);
                    }
                    thread::sleep(Duration::from_millis(25));
                }
            }
        }
    }
}

fn type_text(text: &str) {
    for ch in text.chars() {
        if STOP_TYPING.load(Ordering::SeqCst) {
            return;
        }
        unsafe {
            if let Some((vk, shift)) = char_to_vk(ch) {
                if shift {
                    send_virtual_key(0x10, false);
                }
                send_virtual_key(vk, false);
                send_virtual_key(vk, true);
                if shift {
                    send_virtual_key(0x10, true);
                }
            } else {
                for unit in ch.encode_utf16(&mut [0; 2]) {
                    send_unicode_key(*unit, false);
                    send_unicode_key(*unit, true);
                }
            }
        }
        thread::sleep(Duration::from_millis(25));
    }
}

fn type_literal_text(text: &str) {
    for ch in text.chars() {
        if STOP_TYPING.load(Ordering::SeqCst) {
            return;
        }
        for unit in ch.encode_utf16(&mut [0; 2]) {
            unsafe {
                send_unicode_key(*unit, false);
                send_unicode_key(*unit, true);
            }
            thread::sleep(Duration::from_millis(70));
        }
    }
}

fn press_named_key(key: &str) {
    if let Some(vk) = key_to_vk(key) {
        unsafe {
            send_virtual_key(vk, false);
            thread::sleep(Duration::from_millis(50));
            send_virtual_key(vk, true);
        }
    }
}

fn press_shift_repeat_key(key: &str, count: usize) {
    if let Some(vk) = key_to_vk(key) {
        unsafe {
            send_virtual_key(0x10, false);
            thread::sleep(Duration::from_millis(60));
            for _ in 0..count {
                if STOP_TYPING.load(Ordering::SeqCst) {
                    break;
                }
                send_virtual_key(vk, false);
                thread::sleep(Duration::from_millis(55));
                send_virtual_key(vk, true);
                thread::sleep(Duration::from_millis(110));
            }
            send_virtual_key(0x10, true);
        }
    }
}

fn sleep_interruptible(ms: u64) {
    let mut remaining = ms;
    while remaining > 0 {
        if STOP_TYPING.load(Ordering::SeqCst) {
            return;
        }
        let step = remaining.min(50);
        thread::sleep(Duration::from_millis(step));
        remaining -= step;
    }
}

fn scaled_course_delay(ms: u64) -> u64 {
    ((ms * COURSE_SPEED_PERCENT + 99) / 100).max(15)
}

fn course_sleep(ms: u64) {
    sleep_interruptible(scaled_course_delay(ms));
}

fn spawn_with_program_directory(path: &str) {
    let program = PathBuf::from(path);
    let is_script = matches!(
        program.extension().and_then(|value| value.to_str()),
        Some("bat") | Some("BAT") | Some("cmd") | Some("CMD")
    );
    let mut command = if is_script {
        let mut command = Command::new("cmd.exe");
        command.args(["/D", "/S", "/C", path]);
        command
    } else {
        Command::new(&program)
    };
    if let Some(parent) = program.parent() {
        command.current_dir(parent);
    }
    let _ = command.spawn();
}

fn key_to_vk(key: &str) -> Option<u16> {
    match key.to_ascii_lowercase().as_str() {
        "tab" => Some(0x09),
        "enter" => Some(0x0D),
        "backspace" => Some(0x08),
        "delete" | "del" => Some(0x2E),
        "right" => Some(0x27),
        "left" => Some(0x25),
        "up" => Some(0x26),
        "down" => Some(0x28),
        "space" => Some(0x20),
        "f4" => Some(0x73),
        "esc" | "escape" => Some(0x1B),
        "shift" => Some(0x10),
        "ctrl" | "control" => Some(0x11),
        "alt" => Some(0x12),
        _ => None,
    }
}

unsafe fn send_unicode_key(ch: u16, key_up: bool) {
    let flags = KEYEVENTF_UNICODE | if key_up { KEYEVENTF_KEYUP } else { 0 };
    let input = Input {
        input_type: INPUT_KEYBOARD,
        u: InputUnion {
            ki: KeybdInput {
                w_vk: 0,
                w_scan: ch,
                dw_flags: flags,
                time: 0,
                dw_extra_info: 0,
            },
        },
    };
    SendInput(1, &input, size_of::<Input>() as i32);
}

unsafe fn send_virtual_key(vk: u16, key_up: bool) {
    let scan = MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) as u16;
    let extended = matches!(
        vk,
        0x21 | 0x22 | 0x23 | 0x24 | 0x25 | 0x26 | 0x27 | 0x28 | 0x2D | 0x2E
    );
    let input = Input {
        input_type: INPUT_KEYBOARD,
        u: InputUnion {
            ki: KeybdInput {
                w_vk: 0,
                w_scan: scan,
                dw_flags: KEYEVENTF_SCANCODE
                    | if extended { KEYEVENTF_EXTENDEDKEY } else { 0 }
                    | if key_up { KEYEVENTF_KEYUP } else { 0 },
                time: 0,
                dw_extra_info: 0,
            },
        },
    };
    SendInput(1, &input, size_of::<Input>() as i32);
}

fn char_to_vk(ch: char) -> Option<(u16, bool)> {
    match ch {
        'a'..='z' => Some((ch.to_ascii_uppercase() as u16, false)),
        'A'..='Z' => Some((ch as u16, true)),
        '0'..='9' => Some((ch as u16, false)),
        '.' => Some((0xBE, false)),
        ',' => Some((0xBC, false)),
        '-' => Some((0xBD, false)),
        '/' => Some((0xBF, false)),
        ':' => Some((0xBA, true)),
        ';' => Some((0xBA, false)),
        ' ' => Some((0x20, false)),
        '\n' => Some((0x0D, false)),
        '\t' => Some((0x09, false)),
        _ => None,
    }
}

unsafe fn set_status(text: &str) {
    SetWindowTextW(APP.status, wide(text).as_ptr());
    InvalidateRect(APP.status, null(), 1);
}

unsafe fn apply_font(hwnd: Hwnd, font: Hfont) {
    if hwnd != 0 && font != 0 {
        SendMessageW(hwnd, WM_SETFONT, font as Wparam, 1);
    }
}

// Keep all template-entry waits at the same 2x cadence without affecting app launch automation.
fn template_sleep(ms: u64) {
    sleep_interruptible((ms / 2).max(1));
}

unsafe fn create_font(height: i32, weight: i32) -> Hfont {
    CreateFontW(
        -height,
        0,
        0,
        0,
        weight,
        0,
        0,
        0,
        DEFAULT_CHARSET,
        0,
        0,
        CLEARTYPE_QUALITY,
        0,
        wide("Microsoft YaHei UI").as_ptr(),
    )
}

extern "system" fn keyboard_proc(n_code: i32, w_param: Wparam, l_param: Lparam) -> Lresult {
    unsafe {
        if n_code == HC_ACTION {
            let event = *(l_param as *const KbdllHookStruct);
            let is_down = w_param as u32 == WM_KEYDOWN || w_param as u32 == WM_SYSKEYDOWN;
            let is_up = w_param as u32 == WM_KEYUP || w_param as u32 == WM_SYSKEYUP;

            if matches!(event.vk_code, VK_CONTROL | VK_LCONTROL | VK_RCONTROL) {
                CTRL_DOWN.store(is_down && !is_up, Ordering::SeqCst);
            }
            if matches!(event.vk_code, VK_MENU | VK_LMENU | VK_RMENU) {
                ALT_DOWN.store(is_down && !is_up, Ordering::SeqCst);
            }
            if event.vk_code == VK_RCONTROL {
                RIGHT_CTRL_DOWN.store(is_down && !is_up, Ordering::SeqCst);
            }
            if event.vk_code == VK_RMENU {
                RIGHT_ALT_DOWN.store(is_down && !is_up, Ordering::SeqCst);
            }

            if RIGHT_CTRL_DOWN.load(Ordering::SeqCst) && RIGHT_ALT_DOWN.load(Ordering::SeqCst) {
                STOP_TYPING.store(true, Ordering::SeqCst);
                if APP.status != 0 {
                    set_status("已通过右Ctrl+右Alt停止输入");
                }
            }

            if event.vk_code == VK_V {
                let clipboard_hotkey = checkbox_checked(ID_CLIPBOARD_AUTO)
                    && CTRL_DOWN.load(Ordering::SeqCst)
                    && ALT_DOWN.load(Ordering::SeqCst)
                    && !(RIGHT_CTRL_DOWN.load(Ordering::SeqCst)
                        && RIGHT_ALT_DOWN.load(Ordering::SeqCst));
                if is_down && clipboard_hotkey {
                    if !CLIPBOARD_HOTKEY_DOWN.swap(true, Ordering::SeqCst) {
                        // Capture this before starting the worker. At this
                        // point it is still the window where the user placed
                        // the caret, rather than the assistant's status UI.
                        let target = GetForegroundWindow();
                        // Always refresh the value, including when the
                        // assistant itself is foreground, so a prior target
                        // can never be reused accidentally.
                        CLIPBOARD_TARGET_WINDOW.store(target, Ordering::SeqCst);
                        start_clipboard_typing_flow();
                    }
                    return 1;
                }
                if is_up && CLIPBOARD_HOTKEY_DOWN.swap(false, Ordering::SeqCst) {
                    return 1;
                }
            }

            if event.vk_code == VK_INSERT && checkbox_checked(ID_SHORTCUT_SAVE_ORDER) {
                if is_down && !INSERT_HOTKEY_DOWN.swap(true, Ordering::SeqCst) {
                    start_save_order_flow();
                }
                if is_up {
                    INSERT_HOTKEY_DOWN.store(false, Ordering::SeqCst);
                }
                return 1;
            }

            if event.vk_code == VK_F1 {
                if w_param as u32 == WM_SYSKEYDOWN
                    && !ALT_F1_HOTKEY_DOWN.swap(true, Ordering::SeqCst)
                {
                    start_clinical_path_flow();
                }
                if is_up {
                    ALT_F1_HOTKEY_DOWN.store(false, Ordering::SeqCst);
                }
            }
            if event.vk_code == VK_F2 {
                if w_param as u32 == WM_SYSKEYDOWN
                    && !ALT_F2_HOTKEY_DOWN.swap(true, Ordering::SeqCst)
                {
                    start_clinical_path_continuous_flow();
                }
                if is_up {
                    ALT_F2_HOTKEY_DOWN.store(false, Ordering::SeqCst);
                }
            }
        }
        CallNextHookEx(0, n_code, w_param, l_param)
    }
}

fn loword(value: u32) -> u16 {
    (value & 0xffff) as u16
}

fn hiword(value: u32) -> u16 {
    ((value >> 16) & 0xffff) as u16
}

fn wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(once(0)).collect()
}

fn wide_double_null(parts: &[&str]) -> Vec<u16> {
    let mut output = Vec::new();
    for part in parts {
        output.extend(OsStr::new(part).encode_wide());
        output.push(0);
    }
    output.push(0);
    output
}
