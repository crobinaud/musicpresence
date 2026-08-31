fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("app.ico");
        res.set("ProductName", "MusicPresence");
        res.set("FileDescription", "Apple Music Discord Rich Presence");
        res.set("OriginalFilename", "MusicPresence.exe");
        res.set("InternalName", "MusicPresence.exe");
        res.set("LegalCopyright", "Copyright (C) 2026 Cyprien ROBINAUD.");
        let _ = res.compile();
    }
}
