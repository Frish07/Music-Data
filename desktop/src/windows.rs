use crate::metadata::{self, Fields, Track};
use id3::frame::Picture;
use std::{cell::RefCell, ffi::c_void, mem::{size_of, zeroed}, os::windows::ffi::OsStringExt, path::PathBuf, ptr::{null, null_mut}};
use windows_sys::Win32::{
    Foundation::*, Graphics::{Gdi::*, Dwm::DwmSetWindowAttribute}, System::LibraryLoader::GetModuleHandleW,
    UI::{Controls::{Dialogs::*, *}, HiDpi::*, Input::KeyboardAndMouse::EnableWindow, Shell::*, WindowsAndMessaging::*},
};

const OPEN: usize = 101;
const COVER: usize = 102;
const REMOVE: usize = 103;
const RESET: usize = 104;
const PREVIEW: usize = 105;
const SAVE: usize = 106;
const BACKGROUND: u32 = rgb(20, 22, 22);
const PANEL: u32 = rgb(28, 31, 30);
const INPUT: u32 = rgb(35, 39, 37);
const WHITE: u32 = rgb(233, 239, 234);
const MUTED: u32 = rgb(155, 169, 158);
const ACCENT: u32 = rgb(183, 205, 161);
const fn rgb(r: u8, g: u8, b: u8) -> u32 { r as u32 | ((g as u32) << 8) | ((b as u32) << 16) }
fn wide(text: &str) -> Vec<u16> { text.encode_utf16().chain(Some(0)).collect() }

struct Art { pixels: Vec<u8>, width: i32, height: i32 }
struct App {
    track: Option<Track>, cover: Option<Picture>, art: Option<Art>,
    edits: [HWND; 6], buttons: [HWND; 6], status: HWND,
    brush: HBRUSH, font: HFONT, title_font: HFONT, small_font: HFONT,
    error: bool, dirty: bool, scale: f32,
}
thread_local! { static APP: RefCell<Option<App>> = const { RefCell::new(None) }; }
fn art(picture: &Option<Picture>) -> Option<Art> {
    let image = metadata::decode_cover(&picture.as_ref()?.data).ok()?.thumbnail(480, 480).to_rgba8();
    let (width, height) = image.dimensions();
    let mut pixels = image.into_raw();
    for pixel in pixels.chunks_exact_mut(4) {
        let alpha = pixel[3] as u32;
        for c in pixel.iter_mut().take(3) { *c = ((*c as u32 * alpha + 31 * (255 - alpha)) / 255) as u8; }
        pixel.swap(0, 2);
    }
    Some(Art { pixels, width: width as i32, height: height as i32 })
}

unsafe fn text(hwnd: HWND) -> String {
    let mut buffer = vec![0u16; GetWindowTextLengthW(hwnd) as usize + 1];
    let size = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
    String::from_utf16_lossy(&buffer[..size as usize])
}
unsafe fn fields() -> Fields {
    let edits = APP.with(|a| a.borrow().as_ref().unwrap().edits);
    let values = edits.map(|h| text(h).trim().to_owned());
    Fields { title: values[0].clone(), artist: values[1].clone(), album: values[2].clone(), year: values[3].clone(), genre: values[4].clone(), comment: values[5].clone() }
}
unsafe fn message(hwnd: HWND, content: &str, error: bool) {
    let status = APP.with(|a| { let mut state = a.borrow_mut(); let s = state.as_mut().unwrap(); s.error = error; s.status });
    SetWindowTextW(status, wide(content).as_ptr());
    InvalidateRect(hwnd, null(), 0);
}
unsafe fn confirm_discard(hwnd: HWND) -> bool {
    !APP.with(|a| a.borrow().as_ref().unwrap().dirty) || MessageBoxW(hwnd, wide("Несохранённые изменения будут потеряны. Продолжить?").as_ptr(), wide("Music Data").as_ptr(), MB_YESNO | MB_ICONQUESTION) == IDYES
}
unsafe fn pick(hwnd: HWND, save: bool, cover: bool, name: &str) -> Option<PathBuf> {
    let mut buffer = vec![0u16; 32768];
    let initial = wide(name);
    buffer[..initial.len().min(32767)].copy_from_slice(&initial[..initial.len().min(32767)]);
    let filter = wide(if cover { "Изображения JPG / PNG\0*.jpg;*.jpeg;*.png\0\0" } else { "Аудиофайлы MP3\0*.mp3\0\0" });
    let title = wide(if cover { "Выберите обложку" } else if save { "Сохранить новую копию MP3" } else { "Открыть MP3" });
    let ext = wide(if cover { "png" } else { "mp3" });
    let mut dialog: OPENFILENAMEW = zeroed();
    dialog.lStructSize = size_of::<OPENFILENAMEW>() as u32;
    dialog.hwndOwner = hwnd;
    dialog.lpstrFilter = filter.as_ptr();
    dialog.lpstrFile = buffer.as_mut_ptr();
    dialog.nMaxFile = buffer.len() as u32;
    dialog.lpstrTitle = title.as_ptr();
    dialog.lpstrDefExt = ext.as_ptr();
    dialog.Flags = OFN_EXPLORER | OFN_NOCHANGEDIR | OFN_PATHMUSTEXIST | if save { OFN_OVERWRITEPROMPT } else { OFN_FILEMUSTEXIST };
    let ok = if save { GetSaveFileNameW(&mut dialog) } else { GetOpenFileNameW(&mut dialog) };
    if ok == 0 { return None; }
    let end = buffer.iter().position(|c| *c == 0).unwrap_or(buffer.len());
    Some(PathBuf::from(std::ffi::OsString::from_wide(&buffer[..end])))
}
unsafe fn populate(hwnd: HWND) {
    let (edits, data, buttons) = APP.with(|a| {
        let a = a.borrow(); let s = a.as_ref().unwrap();
        (s.edits, s.track.as_ref().map(|t| t.fields.clone()).unwrap_or_default(), s.buttons)
    });
    for (edit, value) in edits.into_iter().zip([data.title, data.artist, data.album, data.year, data.genre, data.comment]) { SetWindowTextW(edit, wide(&value).as_ptr()); EnableWindow(edit, 1); }
    for button in &buttons[1..] { EnableWindow(*button, 1); }
    APP.with(|a| a.borrow_mut().as_mut().unwrap().dirty = false);
    InvalidateRect(hwnd, null(), 0);
}
unsafe fn load(hwnd: HWND, path: PathBuf) {
    if !confirm_discard(hwnd) { return; }
    match metadata::load(&path) {
        Ok(track) => {
            let cover = track.cover.clone(); let preview = art(&cover);
            APP.with(|a| { let mut a = a.borrow_mut(); let s = a.as_mut().unwrap(); s.track = Some(track); s.cover = cover; s.art = preview; });
            populate(hwnd); message(hwnd, "Теги прочитаны. Измените поля и сохраните новую копию MP3.", false);
        }
        Err(error) => message(hwnd, &error, true),
    }
}
unsafe fn action(hwnd: HWND, id: usize) {
    match id {
        OPEN => if let Some(path) = pick(hwnd, false, false, "") { load(hwnd, path); },
        COVER => if let Some(path) = pick(hwnd, false, true, "") {
            match metadata::cover_from_path(&path) {
                Ok(picture) => { let cover = Some(picture); let preview = art(&cover);
                    APP.with(|a| { let mut a = a.borrow_mut(); let s = a.as_mut().unwrap(); s.cover = cover; s.art = preview; s.dirty = true; });
                    message(hwnd, "Обложка добавлена. Сохраните MP3, чтобы записать изменения.", false);
                }
                Err(error) => message(hwnd, &error, true),
            }
        },
        REMOVE => { APP.with(|a| { let mut a = a.borrow_mut(); let s = a.as_mut().unwrap(); s.cover = None; s.art = None; s.dirty = true; }); message(hwnd, "Обложка будет удалена из новой копии MP3.", false); }
        RESET => {
            if !confirm_discard(hwnd) { return; }
            APP.with(|a| { let mut a = a.borrow_mut(); let s = a.as_mut().unwrap(); s.cover = s.track.as_ref().and_then(|t| t.cover.clone()); s.art = art(&s.cover); });
            populate(hwnd); message(hwnd, "Исходные теги восстановлены.", false);
        }
        PREVIEW => {
            let f = fields();
            let has_cover = APP.with(|a| a.borrow().as_ref().unwrap().cover.is_some());
            MessageBoxW(hwnd, wide(&format!("Название: {}\n\nИсполнитель: {}\n\nАльбом: {}\n\nГод: {}\n\nЖанр: {}\n\nКомментарий: {}\n\nОбложка: {}", f.title, f.artist, f.album, f.year, f.genre, f.comment, if has_cover { "добавлена" } else { "нет" })).as_ptr(), wide("Предпросмотр метаданных").as_ptr(), MB_OK | MB_ICONINFORMATION);
        }
        SAVE => {
            let f = fields();
            let name = APP.with(|a| { let a = a.borrow(); let t = a.as_ref().unwrap().track.as_ref().unwrap(); format!("{} — edited.mp3", t.path.file_stem().unwrap_or_default().to_string_lossy()) });
            if let Some(path) = pick(hwnd, true, false, &name) {
                let result = APP.with(|a| { let a = a.borrow(); let s = a.as_ref().unwrap(); metadata::save_copy(s.track.as_ref().unwrap(), &f, &s.cover, &path) });
                match result {
                    Ok(()) => { APP.with(|a| a.borrow_mut().as_mut().unwrap().dirty = false); message(hwnd, &format!("Сохранено: {}", path.display()), false); }
                    Err(error) => message(hwnd, &error, true),
                }
            }
        }
        _ => {}
    }
}

unsafe fn make_control(hwnd: HWND, class: &str, label: &str, style: u32, id: usize, font: HFONT) -> HWND {
    let child = CreateWindowExW(0, wide(class).as_ptr(), wide(label).as_ptr(), WS_CHILD | WS_VISIBLE | WS_TABSTOP | style, 0, 0, 0, 0, hwnd, id as *mut c_void, GetModuleHandleW(null()), null());
    SendMessageW(child, WM_SETFONT, font as usize, 1); child
}
unsafe fn layout(hwnd: HWND) {
    let mut rect: RECT = zeroed(); GetClientRect(hwnd, &mut rect);
    let (s, edits, buttons, status) = APP.with(|a| { let a = a.borrow(); let a = a.as_ref().unwrap(); (a.scale, a.edits, a.buttons, a.status) });
    let w = (rect.right as f32 / s) as i32; let h = (rect.bottom as f32 / s) as i32;
    let mv = |handle, x: i32, y: i32, width: i32, height: i32| { MoveWindow(handle, (x as f32*s) as i32, (y as f32*s) as i32, (width as f32*s) as i32, (height as f32*s) as i32, 1); };
    mv(buttons[0], 48, 210, 240, 46);
    mv(buttons[1], 568, 186, 205, 32); mv(buttons[2], 568, 224, 205, 28);
    for (i, edit) in edits.iter().take(3).enumerate() { mv(*edit, 470, 280 + i as i32*54, w-514, 34); }
    mv(edits[3], 470, 442, 110, 34); mv(edits[4], 594, 442, w-638, 34); mv(edits[5], 470, 496, w-514, (h-656).max(62));
    mv(buttons[3], 362, h-142, 175, 36); mv(buttons[4], w-405, h-142, 165, 36); mv(buttons[5], w-228, h-142, 184, 36);
    mv(status, 360, h-94, w-404, 34);
    InvalidateRect(hwnd, null(), 1);
}
unsafe fn label(dc: HDC, text: &str, x: i32, y: i32, width: i32, height: i32, color: u32, font: HFONT, s: f32) {
    SelectObject(dc, font); SetTextColor(dc, color); SetBkMode(dc, TRANSPARENT as i32);
    let mut r = RECT { left: (x as f32*s) as i32, top: (y as f32*s) as i32, right: ((x+width) as f32*s) as i32, bottom: ((y+height) as f32*s) as i32 };
    let text = wide(text); DrawTextW(dc, text.as_ptr(), -1, &mut r, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX | DT_END_ELLIPSIS);
}
unsafe fn box_fill(dc: HDC, x: i32, y: i32, width: i32, height: i32, color: u32, s: f32) {
    let r = RECT { left: (x as f32*s) as i32, top: (y as f32*s) as i32, right: ((x+width) as f32*s) as i32, bottom: ((y+height) as f32*s) as i32 };
    let brush = CreateSolidBrush(color); FillRect(dc, &r, brush); DeleteObject(brush);
}
unsafe fn paint(hwnd: HWND) {
    let mut ps: PAINTSTRUCT = zeroed(); let dc = BeginPaint(hwnd, &mut ps);
    let mut rect: RECT = zeroed(); GetClientRect(hwnd, &mut rect);
    APP.with(|a| {
        let a = a.borrow(); let Some(a) = a.as_ref() else { return; }; let s = a.scale;
        let w = (rect.right as f32/s) as i32; let h = (rect.bottom as f32/s) as i32;
        box_fill(dc, 0, 0, w, h, BACKGROUND, s);
        box_fill(dc, 24, 146, 288, h-208, PANEL, s); box_fill(dc, 328, 146, w-352, h-208, PANEL, s);
        label(dc, "musicdata.", 28, 20, 300, 42, WHITE, a.title_font, s);
        label(dc, "DESKTOP  /  PORTABLE", w-255, 32, 230, 26, ACCENT, a.small_font, s);
        label(dc, "Ваша музыка. Ваш порядок.", 28, 82, 700, 32, WHITE, a.title_font, s);
        label(dc, "Редактируйте теги MP3 — без интернета и потери качества.", 28, 119, w-55, 22, MUTED, a.small_font, s);
        label(dc, "Аудиофайл", 48, 170, 240, 28, WHITE, a.font, s);
        label(dc, "Перетащите MP3 в окно", 48, 270, 245, 24, ACCENT, a.small_font, s);
        label(dc, "MP3 · до 150 МБ", 48, 299, 240, 24, MUTED, a.small_font, s);
        box_fill(dc, 76, 348, 180, 180, rgb(44, 53, 40), s);
        if let Some(art) = &a.art {
            let mut bmi: BITMAPINFO = zeroed(); bmi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32; bmi.bmiHeader.biWidth = art.width; bmi.bmiHeader.biHeight = -art.height; bmi.bmiHeader.biPlanes = 1; bmi.bmiHeader.biBitCount = 32; bmi.bmiHeader.biCompression = BI_RGB;
            let ratio = (180.0 / art.width as f32).min(180.0 / art.height as f32);
            let aw = art.width as f32*ratio; let ah = art.height as f32*ratio;
            SetStretchBltMode(dc, HALFTONE as i32);
            StretchDIBits(dc, ((76.0+(180.0-aw)/2.0)*s) as i32, ((348.0+(180.0-ah)/2.0)*s) as i32, (aw*s) as i32, (ah*s) as i32, 0, 0, art.width, art.height, art.pixels.as_ptr().cast(), &bmi, DIB_RGB_COLORS, SRCCOPY);
        } else { label(dc, "♪", 136, 399, 90, 90, ACCENT, a.title_font, s); }
        if let Some(track) = &a.track {
            label(dc, &track.path.file_name().unwrap_or_default().to_string_lossy(), 48, 548, 240, 42, WHITE, a.small_font, s);
            label(dc, &format!("{:.2} МБ · ID3", track.size as f64/1048576.0), 48, 595, 240, 22, MUTED, a.small_font, s);
        }
        label(dc, "Метаданные трека", 352, 167, w-400, 30, WHITE, a.font, s);
        for (name, y) in [("Обложка", 212), ("Название", 286), ("Исполнитель", 340), ("Альбом", 394), ("Год / жанр", 448), ("Комментарий", 503)] {
            label(dc, name, 352, y, 115, 28, MUTED, a.small_font, s);
        }
        label(dc, "JPG / PNG\nдо 3 МБ", 470, 212, 95, 40, ACCENT, a.small_font, s);
        label(dc, "Без установки. Без регистрации. Файлы остаются на вашем устройстве.", 28, h-38, w-55, 25, MUTED, a.small_font, s);
    });
    EndPaint(hwnd, &ps);
}

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if !APP.with(|a| a.borrow().is_some()) { return DefWindowProcW(hwnd, msg, wp, lp); }
    match msg {
        WM_PAINT => { paint(hwnd); 0 }
        WM_ERASEBKGND => 1,
        WM_SIZE => { layout(hwnd); 0 }
        WM_DRAWITEM => {
            let item = &*(lp as *const DRAWITEMSTRUCT);
            let disabled = item.itemState & ODS_DISABLED != 0;
            let pressed = item.itemState & ODS_SELECTED != 0;
            let primary = item.CtlID as usize == SAVE;
            let color = if disabled { rgb(38, 43, 39) } else if primary { if pressed { rgb(29, 78, 180) } else { rgb(36, 103, 237) } } else if pressed { rgb(60, 68, 61) } else { rgb(49, 57, 50) };
            let brush = CreateSolidBrush(color); FillRect(item.hDC, &item.rcItem, brush); DeleteObject(brush);
            SetBkMode(item.hDC, TRANSPARENT as i32); SetTextColor(item.hDC, if disabled { MUTED } else { WHITE });
            let font = APP.with(|a| a.borrow().as_ref().unwrap().font); SelectObject(item.hDC, font);
            let caption = wide(&text(item.hwndItem)); let mut rect = item.rcItem;
            DrawTextW(item.hDC, caption.as_ptr(), -1, &mut rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
            if item.itemState & ODS_FOCUS != 0 { let mut focus = item.rcItem; focus.left += 3; focus.top += 3; focus.right -= 3; focus.bottom -= 3; DrawFocusRect(item.hDC, &focus); }
            1
        }
        WM_GETMINMAXINFO => { let info = &mut *(lp as *mut MINMAXINFO); let s = APP.with(|a| a.borrow().as_ref().unwrap().scale); info.ptMinTrackSize = POINT { x: (940.0*s) as i32, y: (740.0*s) as i32 }; 0 }
        WM_COMMAND => {
            let id = wp & 0xffff; let notification = (wp >> 16) & 0xffff;
            if (OPEN..=SAVE).contains(&id) && notification == BN_CLICKED as usize { action(hwnd, id); }
            else if (200..206).contains(&id) && notification == EN_CHANGE as usize { APP.with(|a| a.borrow_mut().as_mut().unwrap().dirty = true); }
            0
        }
        WM_CTLCOLOREDIT | WM_CTLCOLORSTATIC => APP.with(|a| { let a = a.borrow(); let a = a.as_ref().unwrap(); let dc = wp as HDC;
            SetTextColor(dc, if lp as HWND == a.status && a.error { rgb(255, 160, 150) } else { WHITE });
            SetBkColor(dc, INPUT); a.brush as LRESULT
        }),
        WM_DROPFILES => {
            let drop = wp as HDROP;
            let count = DragQueryFileW(drop, u32::MAX, null_mut(), 0);
            let mut path = vec![0u16; DragQueryFileW(drop, 0, null_mut(), 0) as usize + 1];
            DragQueryFileW(drop, 0, path.as_mut_ptr(), path.len() as u32); DragFinish(drop);
            if count == 1 { path.pop(); load(hwnd, PathBuf::from(std::ffi::OsString::from_wide(&path))); }
            else { message(hwnd, "Добавляйте по одному MP3 за раз.", true); } 0
        }
        WM_TIMER => { KillTimer(hwnd, 1); PostMessageW(hwnd, WM_CLOSE, 0, 0); 0 }
        WM_DPICHANGED => {
            let r = &*(lp as *const RECT); SetWindowPos(hwnd, null_mut(), r.left, r.top, r.right-r.left, r.bottom-r.top, SWP_NOZORDER | SWP_NOACTIVATE);
            // Windows scales the current fonts; layout stays consistent with its initial logical scale.
            layout(hwnd); 0
        }
        WM_CLOSE => { if confirm_discard(hwnd) { DestroyWindow(hwnd); } 0 }
        WM_DESTROY => { PostQuitMessage(0); 0 }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

pub fn run() { unsafe {
    SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    let instance = GetModuleHandleW(null()); let class = wide("MusicDataPortable");
    let wc = WNDCLASSW { style: CS_HREDRAW | CS_VREDRAW, lpfnWndProc: Some(window_proc), hInstance: instance, hCursor: LoadCursorW(null_mut(), IDC_ARROW), hIcon: LoadIconW(null_mut(), IDI_APPLICATION), lpszClassName: class.as_ptr(), ..zeroed() };
    RegisterClassW(&wc);
    let hwnd = CreateWindowExW(WS_EX_ACCEPTFILES, class.as_ptr(), wide("Music Data — редактор метаданных").as_ptr(), WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN, CW_USEDEFAULT, CW_USEDEFAULT, 1120, 820, null_mut(), null_mut(), instance, null());
    if hwnd.is_null() { std::process::exit(1); }
    let dark: i32 = 1;
    DwmSetWindowAttribute(hwnd, 20, (&dark as *const i32).cast(), size_of::<i32>() as u32);
    let scale = GetDpiForWindow(hwnd) as f32/96.0;
    let font = |size: f32| CreateFontW(-(size*scale) as i32, 0, 0, 0, 400, 0, 0, 0, DEFAULT_CHARSET as u32, OUT_DEFAULT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32, CLEARTYPE_QUALITY as u32, DEFAULT_PITCH as u32, wide("Segoe UI").as_ptr());
    let normal = font(16.0); let title = font(27.0); let small = font(14.0);
    let edits = std::array::from_fn(|i| {
        let style = WS_BORDER | if i == 5 { ES_MULTILINE as u32 | ES_AUTOVSCROLL as u32 | ES_WANTRETURN as u32 | WS_VSCROLL } else { ES_AUTOHSCROLL as u32 };
        let h = make_control(hwnd, "EDIT", "", style, 200+i, normal);
        SendMessageW(h, EM_SETLIMITTEXT, if i == 5 { 5000 } else if i == 3 { 4 } else { 500 }, 0);
        SendMessageW(h, EM_SETCUEBANNER, 0, wide(["Название композиции", "Имя исполнителя или группы", "Название альбома", "Год", "Жанр", "Короткий комментарий…"][i]).as_ptr() as isize);
        EnableWindow(h, 0); h
    });
    let buttons = ["Открыть MP3…", "Выбрать изображение…", "Удалить обложку", "Сбросить изменения", "Предпросмотр", "Сохранить MP3…"].map(|name| make_control(hwnd, "BUTTON", name, BS_OWNERDRAW as u32, 0, normal));
    for (i, button) in buttons.iter().enumerate() { SetWindowLongPtrW(*button, GWLP_ID, (OPEN+i) as isize); if i != 0 { EnableWindow(*button, 0); } }
    // SS_LEFT is the zero-valued default style for a STATIC control.
    let status = make_control(hwnd, "STATIC", "Выберите MP3, чтобы изменить его метаданные.", 0, 300, small);
    APP.with(|a| *a.borrow_mut() = Some(App { track: None, cover: None, art: None, edits, buttons, status, brush: CreateSolidBrush(INPUT), font: normal, title_font: title, small_font: small, error: false, dirty: false, scale }));
    let mut work: RECT = zeroed(); SystemParametersInfoW(SPI_GETWORKAREA, 0, (&mut work as *mut RECT).cast(), 0);
    let width = (1120.0*scale) as i32; let height = (820.0*scale) as i32;
    SetWindowPos(hwnd, null_mut(), work.left, work.top, width.min(work.right-work.left), height.min(work.bottom-work.top), SWP_NOZORDER | SWP_NOACTIVATE);
    layout(hwnd); ShowWindow(hwnd, SW_SHOW); UpdateWindow(hwnd); DragAcceptFiles(hwnd, 1);
    let smoke = std::env::args().any(|a| a == "--smoke-test");
    if smoke { SetTimer(hwnd, 1, 500, None); }
    else if let Some(path) = std::env::args_os().nth(1) { load(hwnd, PathBuf::from(path)); }
    let mut msg: MSG = zeroed();
    while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
        if IsDialogMessageW(hwnd, &msg) == 0 { TranslateMessage(&msg); DispatchMessageW(&msg); }
    }
    APP.with(|a| { if let Some(a) = a.borrow_mut().take() { DeleteObject(a.brush); DeleteObject(a.font); DeleteObject(a.title_font); DeleteObject(a.small_font); } });
} }
