param(
    [int]$Rate = 2,
    [int]$Volume = 100
)

$ErrorActionPreference = "Stop"

if ([System.Threading.Thread]::CurrentThread.GetApartmentState() -ne [System.Threading.ApartmentState]::STA) {
    Start-Process -FilePath "powershell.exe" -ArgumentList @(
        "-NoProfile",
        "-ExecutionPolicy", "Bypass",
        "-STA",
        "-File", "`"$PSCommandPath`"",
        "-Rate", $Rate,
        "-Volume", $Volume
    ) -WindowStyle Hidden
    exit
}

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$source = @"
using System;
using System.Runtime.InteropServices;
using System.Windows.Forms;

namespace ReadThis {
    public sealed class HotkeyWindow : Form {
        public event Action<int> HotkeyPressed;

        private const int WM_HOTKEY = 0x0312;
        private const int MOD_ALT = 0x0001;
        private const int MOD_CONTROL = 0x0002;
        private const int MOD_NOREPEAT = 0x4000;
        private const int VK_R = 0x52;
        private const int VK_S = 0x53;
        private const int VK_CONTROL = 0x11;
        private const int VK_MENU = 0x12;

        [DllImport("user32.dll", SetLastError = true)]
        private static extern bool RegisterHotKey(IntPtr hWnd, int id, int fsModifiers, int vk);

        [DllImport("user32.dll", SetLastError = true)]
        private static extern bool UnregisterHotKey(IntPtr hWnd, int id);

        [DllImport("user32.dll")]
        private static extern short GetAsyncKeyState(int vKey);

        public static bool IsModifierPressed() {
            return (GetAsyncKeyState(VK_CONTROL) & 0x8000) != 0 ||
                   (GetAsyncKeyState(VK_MENU) & 0x8000) != 0;
        }

        public static bool IsCtrlAltKeyPressed(int key) {
            return (GetAsyncKeyState(VK_CONTROL) & 0x8000) != 0 &&
                   (GetAsyncKeyState(VK_MENU) & 0x8000) != 0 &&
                   (GetAsyncKeyState(key) & 0x8000) != 0;
        }

        protected override void OnHandleCreated(EventArgs e) {
            base.OnHandleCreated(e);
            RegisterHotKey(this.Handle, 1, MOD_CONTROL | MOD_ALT | MOD_NOREPEAT, VK_R);
            RegisterHotKey(this.Handle, 2, MOD_CONTROL | MOD_ALT | MOD_NOREPEAT, VK_S);
        }

        protected override void OnHandleDestroyed(EventArgs e) {
            UnregisterHotKey(this.Handle, 1);
            UnregisterHotKey(this.Handle, 2);
            base.OnHandleDestroyed(e);
        }

        protected override void SetVisibleCore(bool value) {
            base.SetVisibleCore(false);
        }

        protected override void WndProc(ref Message m) {
            if (m.Msg == WM_HOTKEY) {
                if (HotkeyPressed != null) {
                    HotkeyPressed(m.WParam.ToInt32());
                }
                return;
            }

            base.WndProc(ref m);
        }
    }
}
"@

Add-Type -TypeDefinition $source -ReferencedAssemblies "System.Windows.Forms.dll"

$voice = New-Object -ComObject SAPI.SpVoice
$voice.Rate = [Math]::Max(-10, [Math]::Min(10, $Rate))
$voice.Volume = [Math]::Max(0, [Math]::Min(100, $Volume))

$notifyIcon = New-Object System.Windows.Forms.NotifyIcon
$notifyIcon.Text = "Readtis - Ctrl+Alt+R reads, Ctrl+Alt+S stops"
$notifyIcon.Icon = [System.Drawing.SystemIcons]::Information
$notifyIcon.Visible = $true

$menu = New-Object System.Windows.Forms.ContextMenuStrip
$readItem = New-Object System.Windows.Forms.ToolStripMenuItem
$readItem.Text = "Read highlighted text"
$testItem = New-Object System.Windows.Forms.ToolStripMenuItem
$testItem.Text = "Test voice"
$stopItem = New-Object System.Windows.Forms.ToolStripMenuItem
$stopItem.Text = "Stop"
$exitItem = New-Object System.Windows.Forms.ToolStripMenuItem
$exitItem.Text = "Exit"
$menu.Items.Add($readItem) | Out-Null
$menu.Items.Add($testItem) | Out-Null
$menu.Items.Add($stopItem) | Out-Null
$menu.Items.Add((New-Object System.Windows.Forms.ToolStripSeparator)) | Out-Null
$menu.Items.Add($exitItem) | Out-Null
$notifyIcon.ContextMenuStrip = $menu

function Show-ReadThisTip {
    param(
        [string]$Title,
        [string]$Message,
        [System.Windows.Forms.ToolTipIcon]$Icon = [System.Windows.Forms.ToolTipIcon]::Info
    )

    $notifyIcon.BalloonTipTitle = $Title
    $notifyIcon.BalloonTipText = $Message
    $notifyIcon.BalloonTipIcon = $Icon
    $notifyIcon.ShowBalloonTip(1800)
}

function Get-SelectedText {
    $savedClipboard = $null
    $hadClipboard = $false

    try {
        $savedClipboard = [System.Windows.Forms.Clipboard]::GetDataObject()
        $hadClipboard = $null -ne $savedClipboard
    } catch {
        $hadClipboard = $false
    }

    try {
        for ($i = 0; $i -lt 20 -and [ReadThis.HotkeyWindow]::IsModifierPressed(); $i++) {
            Start-Sleep -Milliseconds 50
        }

        [System.Windows.Forms.Clipboard]::Clear()
        Start-Sleep -Milliseconds 50
        [System.Windows.Forms.SendKeys]::SendWait("^c")

        for ($i = 0; $i -lt 12; $i++) {
            Start-Sleep -Milliseconds 80
            if ([System.Windows.Forms.Clipboard]::ContainsText()) {
                $copiedText = [System.Windows.Forms.Clipboard]::GetText()
                if (-not [string]::IsNullOrWhiteSpace($copiedText)) {
                    return $copiedText
                }
            }
        }

        return ""
    } finally {
        if ($hadClipboard) {
            try {
                [System.Windows.Forms.Clipboard]::SetDataObject($savedClipboard, $true)
            } catch {
                # Some apps lock the clipboard briefly. Leaving the copied text is better than crashing.
            }
        }
    }
}

function Start-ReadingSelection {
    try {
        $text = Get-SelectedText
        if ([string]::IsNullOrWhiteSpace($text)) {
            Show-ReadThisTip "Readtis" "No highlighted text was detected." ([System.Windows.Forms.ToolTipIcon]::Warning)
            return
        }

        $null = $voice.Speak("", 2)
        $null = $voice.Speak($text, 1)
        Show-ReadThisTip "Readtis" "Reading highlighted text."
    } catch {
        Show-ReadThisTip "Readtis error" $_.Exception.Message ([System.Windows.Forms.ToolTipIcon]::Error)
    }
}

function Stop-Reading {
    try {
        $null = $voice.Speak("", 2)
        Show-ReadThisTip "Readtis" "Stopped."
    } catch {
        Show-ReadThisTip "Readtis error" $_.Exception.Message ([System.Windows.Forms.ToolTipIcon]::Error)
    }
}

$readItem.add_Click({
    Start-ReadingSelection
})

$testItem.add_Click({
    $null = $voice.Speak("", 2)
    $null = $voice.Speak("Readtis is ready.", 1)
})

$stopItem.add_Click({
    Stop-Reading
})

$form = New-Object ReadThis.HotkeyWindow
$form.add_HotkeyPressed({
    param($id)

    if ($id -eq 1) {
        Start-ReadingSelection
    } elseif ($id -eq 2) {
        Stop-Reading
    }
})

$lastReadChord = $false
$lastStopChord = $false
$hotkeyPoller = New-Object System.Windows.Forms.Timer
$hotkeyPoller.Interval = 80
$hotkeyPoller.add_Tick({
    $readChord = [ReadThis.HotkeyWindow]::IsCtrlAltKeyPressed(0x52)
    $stopChord = [ReadThis.HotkeyWindow]::IsCtrlAltKeyPressed(0x53)

    if ($readChord -and -not $script:lastReadChord) {
        Start-ReadingSelection
    }

    if ($stopChord -and -not $script:lastStopChord) {
        Stop-Reading
    }

    $script:lastReadChord = $readChord
    $script:lastStopChord = $stopChord
})
$hotkeyPoller.Start()

$exitItem.add_Click({
    $hotkeyPoller.Stop()
    $hotkeyPoller.Dispose()
    $notifyIcon.Visible = $false
    $notifyIcon.Dispose()
    [System.Windows.Forms.Application]::Exit()
})

Show-ReadThisTip "Readtis is running" "Highlight text, press Ctrl+Alt+R to read it aloud. Press Ctrl+Alt+S to stop."
[System.Windows.Forms.Application]::Run($form)
