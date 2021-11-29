using System;
using System.Collections.Generic;
using System.Drawing;
using System.Linq;
using System.Net;
using System.Net.Sockets;
using System.Runtime.InteropServices;
using System.Text;
using System.Windows.Forms;

using BizHawk.Client.Common;
using BizHawk.Client.EmuHawk;

namespace Net.Fenhl.OotAutoTracker {
    internal class Native {
        [DllImport("oottracker")] internal static extern StringHandle expected_bizhawk_version_string();
        [DllImport("oottracker")] internal static extern StringHandle running_bizhawk_version_string();
        [DllImport("oottracker")] internal static extern StringHandle version_string();
        [DllImport("oottracker")] internal static extern BoolResult update_available();
        [DllImport("oottracker")] internal static extern void bool_result_free(IntPtr bool_res);
        [DllImport("oottracker")] internal static extern bool bool_result_is_ok(BoolResult bool_res);
        [DllImport("oottracker")] internal static extern bool bool_result_unwrap(IntPtr bool_res);
        [DllImport("oottracker")] internal static extern StringHandle bool_result_debug_err(IntPtr bool_res);
        [DllImport("oottracker")] internal static extern UnitResult run_updater();
        [DllImport("oottracker")] internal static extern TrackerLayout layout_default();
        [DllImport("oottracker")] internal static extern void layout_free(IntPtr layout);
        [DllImport("oottracker")] internal static extern TrackerCell layout_cell(TrackerLayout layout, byte idx);
        [DllImport("oottracker")] internal static extern void cell_free(IntPtr cell);
        [DllImport("oottracker")] internal static extern StringHandle cell_image(ModelState model, TrackerCell cell);
        [DllImport("oottracker")] internal static extern TcpStreamResult connect_ipv4(byte[] addr);
        [DllImport("oottracker")] internal static extern TcpStreamResult connect_ipv6(byte[] addr);
        [DllImport("oottracker")] internal static extern void tcp_stream_result_free(IntPtr tcp_stream_res);
        [DllImport("oottracker")] internal static extern bool tcp_stream_result_is_ok(TcpStreamResult tcp_stream_res);
        [DllImport("oottracker")] internal static extern TcpStream tcp_stream_result_unwrap(IntPtr tcp_stream_res);
        [DllImport("oottracker")] internal static extern void tcp_stream_free(IntPtr tcp_stream);
        [DllImport("oottracker")] internal static extern StringHandle tcp_stream_result_debug_err(IntPtr tcp_stream_res);
        [DllImport("oottracker")] internal static extern void string_free(IntPtr s);
        [DllImport("oottracker")] internal static extern UnitResult tcp_stream_disconnect(IntPtr tcp_stream);
        [DllImport("oottracker")] internal static extern void unit_result_free(IntPtr unit_res);
        [DllImport("oottracker")] internal static extern bool unit_result_is_ok(UnitResult unit_res);
        [DllImport("oottracker")] internal static extern StringHandle unit_result_debug_err(IntPtr unit_res);
        [DllImport("oottracker")] internal static extern SaveResult save_from_save_data(byte[] start);
        [DllImport("oottracker")] internal static extern void save_result_free(IntPtr save_res);
        [DllImport("oottracker")] internal static extern bool save_result_is_ok(SaveResult save_res);
        [DllImport("oottracker")] internal static extern Save save_result_unwrap(IntPtr save_res);
        [DllImport("oottracker")] internal static extern Save save_default();
        [DllImport("oottracker")] internal static extern void save_free(IntPtr save);
        [DllImport("oottracker")] internal static extern StringHandle save_debug(Save save);
        [DllImport("oottracker")] internal static extern StringHandle save_result_debug_err(IntPtr save_res);
        [DllImport("oottracker")] internal static extern UnitResult save_send(TcpStream tcp_stream, Save save);
        [DllImport("oottracker")] internal static extern bool saves_equal(Save save1, Save save2);
        [DllImport("oottracker")] internal static extern SavesDiff saves_diff(Save old_save, Save new_save);
        [DllImport("oottracker")] internal static extern void saves_diff_free(IntPtr diff);
        [DllImport("oottracker")] internal static extern UnitResult saves_diff_send(TcpStream tcp_stream, IntPtr diff);
        [DllImport("oottracker")] internal static extern Knowledge knowledge_none();
        [DllImport("oottracker")] internal static extern Knowledge knowledge_vanilla();
        [DllImport("oottracker")] internal static extern void knowledge_free(IntPtr knowledge);
        [DllImport("oottracker")] internal static extern UnitResult knowledge_send(TcpStream tcp_stream, Knowledge knowledge);
        [DllImport("oottracker")] internal static extern ModelState model_new(IntPtr save, IntPtr knowledge);
        [DllImport("oottracker")] internal static extern void model_free(IntPtr model);
        [DllImport("oottracker")] internal static extern byte ram_num_ranges();
        [DllImport("oottracker")] internal static extern IntPtr ram_ranges();
        [DllImport("oottracker")] internal static extern RamResult ram_from_ranges(IntPtr[] ranges);
        [DllImport("oottracker")] internal static extern void ram_result_free(IntPtr ram_res);
        [DllImport("oottracker")] internal static extern bool ram_result_is_ok(RamResult ram_res);
        [DllImport("oottracker")] internal static extern Ram ram_result_unwrap(IntPtr ram_res);
        [DllImport("oottracker")] internal static extern StringHandle ram_result_debug_err(IntPtr ram_res);
        [DllImport("oottracker")] internal static extern void ram_free(IntPtr ram);
        [DllImport("oottracker")] internal static extern bool ram_equal(Ram ram1, Ram ram2);
        [DllImport("oottracker")] internal static extern void model_set_ram(ModelState model, Ram ram);
        [DllImport("oottracker")] internal static extern Save ram_clone_save(Ram ram);
    }

    internal class StringHandle : SafeHandle {
        internal StringHandle() : base(IntPtr.Zero, true) {}

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        public string AsString() {
            int len = 0;
            while (Marshal.ReadByte(this.handle, len) != 0) { ++len; }
            byte[] buffer = new byte[len];
            Marshal.Copy(this.handle, buffer, 0, buffer.Length);
            return Encoding.UTF8.GetString(buffer);
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.string_free(this.handle);
            }
            return true;
        }
    }

    internal class BoolResult : SafeHandle {
        internal BoolResult() : base(IntPtr.Zero, true) {}

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.bool_result_free(handle);
            }
            return true;
        }

        internal bool IsOk() => Native.bool_result_is_ok(this);

        internal bool Unwrap() {
            var inner = Native.bool_result_unwrap(this.handle);
            this.handle = IntPtr.Zero; // bool_result_unwrap takes ownership
            return inner;
        }

        internal StringHandle DebugErr() {
            var err = Native.bool_result_debug_err(this.handle);
            this.handle = IntPtr.Zero; // bool_result_debug_err takes ownership
            return err;
        }
    }

    internal class TrackerLayout : SafeHandle {
        internal TrackerLayout() : base(IntPtr.Zero, true) {}

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.layout_free(this.handle);
            }
            return true;
        }

        internal TrackerCell Cell(byte idx) => Native.layout_cell(this, idx);
    }

    internal class TrackerCell : SafeHandle {
        internal TrackerCell() : base(IntPtr.Zero, true) {}

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.cell_free(this.handle);
            }
            return true;
        }

        public StringHandle Image(ModelState model) => Native.cell_image(model, this);
    }

    internal class TcpStreamResult : SafeHandle {
        internal TcpStreamResult() : base(IntPtr.Zero, true) {}

        internal static TcpStreamResult Connect(IPAddress addr) {
            return addr.AddressFamily switch {
                AddressFamily.InterNetwork => Native.connect_ipv4(addr.GetAddressBytes().ToArray()),
                AddressFamily.InterNetworkV6 => Native.connect_ipv6(addr.GetAddressBytes().ToArray()),
                _ => throw new NotImplementedException("can only connect to an IPv4 or IPv6 address"),
            };
        }

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.tcp_stream_result_free(this.handle);
            }
            return true;
        }

        internal bool IsOk() => Native.tcp_stream_result_is_ok(this);

        internal TcpStream Unwrap() {
            var tcp_stream = Native.tcp_stream_result_unwrap(this.handle);
            this.handle = IntPtr.Zero; // tcp_stream_result_unwrap takes ownership
            return tcp_stream;
        }

        internal StringHandle DebugErr() {
            var err = Native.tcp_stream_result_debug_err(this.handle);
            this.handle = IntPtr.Zero; // tcp_stream_result_debug_err takes ownership
            return err;
        }
    }

    internal class TcpStream : SafeHandle {
        internal TcpStream() : base(IntPtr.Zero, true) {}

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.tcp_stream_free(this.handle);
            }
            return true;
        }

        internal UnitResult Disconnect() {
            var unit_res = Native.tcp_stream_disconnect(this.handle);
            this.handle = IntPtr.Zero; // tcp_stream_disconnect takes ownership
            return unit_res;
        }
    }

    internal class UnitResult : SafeHandle {
        internal UnitResult() : base(IntPtr.Zero, true) {}

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.unit_result_free(handle);
            }
            return true;
        }

        internal bool IsOk() => Native.unit_result_is_ok(this);

        internal StringHandle DebugErr() {
            var err = Native.unit_result_debug_err(this.handle);
            this.handle = IntPtr.Zero; // unit_result_debug_err takes ownership
            return err;
        }
    }

    internal class SaveResult : SafeHandle {
        internal SaveResult() : base(IntPtr.Zero, true) {}

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.save_result_free(this.handle);
            }
            return true;
        }

        internal Save Unwrap() {
            var save = Native.save_result_unwrap(this.handle);
            this.handle = IntPtr.Zero; // save_result_unwrap takes ownership
            return save;
        }

        internal bool IsOk() => Native.save_result_is_ok(this);

        internal StringHandle DebugErr() {
            var err = Native.save_result_debug_err(this.handle);
            this.handle = IntPtr.Zero; // save_result_debug_err takes ownership
            return err;
        }
    }

    internal class Save : SafeHandle {
        internal Save() : base(IntPtr.Zero, true) {}

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.save_free(this.handle);
            }
            return true;
        }

        public IntPtr Move() {
            var ptr = this.handle;
            this.handle = IntPtr.Zero;
            return ptr;
        }

        internal bool Equals(Save other) => Native.saves_equal(this, other);
        internal SavesDiff Diff(Save other) => Native.saves_diff(this, other);
        internal UnitResult Send(TcpStream tcp_stream) => Native.save_send(tcp_stream, this);
        internal StringHandle Debug() => Native.save_debug(this);
    }

    internal class SavesDiff : SafeHandle {
        internal SavesDiff() : base(IntPtr.Zero, true) {}

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.saves_diff_free(this.handle);
            }
            return true;
        }

        internal UnitResult Send(TcpStream tcp_stream) {
            var unit_res = Native.saves_diff_send(tcp_stream, this.handle);
            this.handle = IntPtr.Zero; // saves_diff_send takes ownership
            return unit_res;
        }
    }

    internal class Knowledge : SafeHandle {
        internal Knowledge() : base(IntPtr.Zero, true) {}

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.knowledge_free(this.handle);
            }
            return true;
        }

        public IntPtr Move() {
            var ptr = this.handle;
            this.handle = IntPtr.Zero;
            return ptr;
        }

        internal UnitResult Send(TcpStream tcp_stream) => Native.knowledge_send(tcp_stream, this);
    }

    internal class ModelState : SafeHandle {
        internal ModelState() : base(IntPtr.Zero, true) {}

        internal static ModelState FromSaveAndKnowledge(Save save, Knowledge knowledge) {
            var save_ptr = save.Move();
            var knowledge_ptr = knowledge.Move();
            return Native.model_new(save_ptr, knowledge_ptr);
        }

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.model_free(this.handle);
            }
            return true;
        }

        public void SetRam(Ram ram) => Native.model_set_ram(this, ram);
    }

    internal class RamResult : SafeHandle {
        internal RamResult() : base(IntPtr.Zero, true) {}

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.ram_result_free(this.handle);
            }
            return true;
        }

        internal bool IsOk() => Native.ram_result_is_ok(this);

        internal Ram Unwrap() {
            var ram = Native.ram_result_unwrap(this.handle);
            this.handle = IntPtr.Zero; // ram_result_unwrap takes ownership
            return ram;
        }

        internal StringHandle DebugErr() {
            var err = Native.ram_result_debug_err(this.handle);
            this.handle = IntPtr.Zero; // ram_result_debug_err takes ownership
            return err;
        }
    }

    internal class Ram : SafeHandle {
        internal Ram() : base(IntPtr.Zero, true) {}

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.ram_free(this.handle);
            }
            return true;
        }

        public IntPtr Move() {
            var ptr = this.handle;
            this.handle = IntPtr.Zero;
            return ptr;
        }

        internal Save CloneSave() => Native.ram_clone_save(this);
        internal bool Equals(Ram other) => Native.ram_equal(this, other);
    }

    class RawRam {
        internal byte num_ranges;
        internal int[] ranges;
        private string[] range_hashes;
        internal byte[][] range_data;

        internal RawRam(IMemoryApi memoryApi) {
            this.num_ranges = Native.ram_num_ranges();
            this.ranges = new int[2 * num_ranges];
            Marshal.Copy(Native.ram_ranges(), this.ranges, 0, 2 * this.num_ranges);
            this.range_hashes = new string[this.num_ranges];
            this.range_data = new byte[this.num_ranges][];
            for (byte i = 0; i < this.num_ranges; i++) {
                this.range_hashes[i] = memoryApi.HashRegion(this.ranges[2 * i], this.ranges[2 * i + 1], "RDRAM");
                this.range_data[i] = memoryApi.ReadByteRange(this.ranges[2 * i], this.ranges[2 * i + 1], "RDRAM").ToArray();
            }
        }

        internal bool Update(IMemoryApi memoryApi) {
            bool changed = false;
            for (byte i = 0; i < this.num_ranges; i++) {
                var new_hash = memoryApi.HashRegion(this.ranges[2 * i], this.ranges[2 * i + 1], "RDRAM");
                if (new_hash != this.range_hashes[i]) {
                    changed = true;
                    this.range_hashes[i] = new_hash;
                    this.range_data[i] = memoryApi.ReadByteRange(this.ranges[2 * i], this.ranges[2 * i + 1], "RDRAM").ToArray();
                }
            }
            return changed;
        }

        internal RamResult ToRam() {
            IntPtr[] range_data = new IntPtr[this.num_ranges];
            for (byte i = 0; i < this.num_ranges; i++) {
                range_data[i] = Marshal.AllocHGlobal(this.ranges[2 * i + 1]);
                Marshal.Copy(this.range_data[i], 0, range_data[i], this.ranges[2 * i + 1]);
            }
            var ram_res = Native.ram_from_ranges(range_data);
            for (byte i = 0; i < this.num_ranges; i++) {
                Marshal.FreeHGlobal(range_data[i]);
            }
            return ram_res;
        }
    }

    [ExternalTool("OoT autotracker", Description = "An auto-tracking plugin for Fenhl's OoT tracker")]
    [ExternalToolEmbeddedIcon("Net.Fenhl.OotAutoTracker.Resources.icon.ico")]
    public sealed class MainForm : ToolFormBase, IExternalToolForm {
        private PictureBox[] cells = new PictureBox[52];
        private Label label_Version = new Label();
        private Button button_Update = new Button();
        private Label label_Update = new Label();
        private Label label_Game = new Label();
        //private Label label_Connection = new Label();
        private Label label_Save = new Label();
        private Label label_Help = new Label();
        private Button button_Close_Menu = new Button();

        public ApiContainer? _apiContainer { get; set; }
        private ApiContainer APIs => _apiContainer ?? throw new NullReferenceException();

        public override bool BlocksInputWhenFocused { get; } = false;
        protected override string WindowTitleStatic => "OoT autotracker";

        public override bool AskSaveChanges() => true;

        private bool isVanilla;
        //private TcpStream? stream;
        private RawRam? rawRam;
        private Ram? prevRam;
        private List<byte> prevSaveData = new List<byte>();
        private Save? prevSave;
        private ModelState model = ModelState.FromSaveAndKnowledge(Native.save_default(), Native.knowledge_none());
        private TrackerLayout layout = Native.layout_default();
        private string[] cellImages = new string[52];

        private bool gameOk = false;
        //private bool connectionOk = false;
        private bool saveOk = false;

        public MainForm() {
            SuspendLayout();
            this.FormBorderStyle = FormBorderStyle.FixedSingle;
            this.MaximizeBox = false;
            this.ClientSize = new Size(720, 896);
            this.Icon = new Icon(typeof(MainForm).Assembly.GetManifestResourceStream("Net.Fenhl.OotAutoTracker.Resources.icon.ico"));
            this.BackColor = Color.Black;
            this.AutoScaleMode = AutoScaleMode.Dpi;

            // cells
            for (int i = 0; i < 52; i++) {
                PictureBox cell = new PictureBox();
                this.cells[i] = cell;
                cell.Location = i switch {
                    _ when i < 6 => new Point(120 * i + 10, 10),
                    _ when i < 14 => new Point(120 * (i % 6) + 10, 120 * (i / 6) - 54),
                    _ when i < 17 => new Point(80 * (i - 14) + 250, 186),
                    _ when i < 19 => new Point(120 * ((i - 1) % 6) + 10, 120 * ((i - 1) / 6) - 54),
                    _ when i < 22 => new Point(80 * (i - 19) + 250, 226),
                    _ => new Point(120 * ((i - 4) % 6) + 10, 120 * ((i - 4) / 6) - 54),
                };
                cell.Size = i switch {
                    _ when i < 6 => new Size(100, 36),
                    14 or 15 or 16 => new Size(60, 20),
                    19 or 20 or 21 => new Size(60, 60),
                    _ => new Size(100, 100),
                };
                cell.SizeMode = PictureBoxSizeMode.StretchImage;
                //TODO accessibility metadata?
                if (i >= 6 && i < 12) {
                    cell.Click += new EventHandler((object sender, EventArgs e) => {
                        MouseEventArgs me = (MouseEventArgs) e;
                        if (me.Button == MouseButtons.Right) {
                            this.label_Version.Visible = true;
                            this.button_Update.Visible = true;
                            this.label_Update.Visible = true;
                            this.label_Game.Visible = true;
                            //this.label_Connection.Visible = true;
                            this.label_Save.Visible = true;
                            this.label_Help.Visible = true;
                            this.button_Close_Menu.Visible = true;
                            foreach (PictureBox cell in this.cells) {
                                cell.Visible = false;
                            }
                            this.FormBorderStyle = FormBorderStyle.Sizable;
                            this.MaximizeBox = true;
                        }
                    });
                }
                this.Controls.Add(cell);
            }
            UpdateCells();

            // label_Version
            this.label_Version.ForeColor = Color.White;
            this.label_Version.AutoSize = true;
            this.label_Version.Location = new Point(12, 9);
            this.label_Version.Name = "label_Version";
            this.label_Version.Size = new Size(96, 25);
            this.label_Version.TabIndex = 0;
            this.label_Version.Text = $"OoT autotracker version {Native.version_string().AsString()} for BizHawk version {Native.expected_bizhawk_version_string().AsString()}";
            this.label_Version.Visible = false;
            this.Controls.Add(this.label_Version);

            // button_Update
            this.button_Update.ForeColor = Color.White;
            this.button_Update.AutoSize = true;
            this.button_Update.Location = new Point(12, 34);
            this.button_Update.Name = "button_Update";
            this.button_Update.Size = new Size(96, 25);
            this.button_Update.TabIndex = 1;
            this.button_Update.Text = "Check for updates…";
            this.button_Update.Visible = false;
            this.button_Update.Click += new EventHandler((object sender, EventArgs e) => {
                this.label_Update.Text = "Checking for updates…";
                using (var update_available_res = Native.update_available()) {
                    if (update_available_res.IsOk()) {
                        if (update_available_res.Unwrap()) {
                            this.label_Update.Text = "An update is available";
                            using (var run_updater_res = Native.run_updater()) {
                                if (!run_updater_res.IsOk()) {
                                    this.label_Update.Text = run_updater_res.DebugErr().AsString();
                                }
                            }
                        } else {
                            this.label_Update.Text = $"You are up to date as of {DateTime.Now}";
                        }
                    } else {
                        this.label_Update.Text = update_available_res.DebugErr().AsString();
                    }
                }
            });
            this.Controls.Add(this.button_Update);

            // label_Update
            this.label_Update.ForeColor = Color.White;
            this.label_Update.AutoSize = true;
            this.label_Update.Location = new Point(222, 39);
            this.label_Update.Name = "label_Update";
            this.label_Update.Size = new Size(96, 25);
            this.label_Update.TabIndex = 2;
            this.label_Update.Text = "";
            this.label_Update.Visible = false;
            this.Controls.Add(this.label_Update);

            // label_Game
            this.label_Game.ForeColor = Color.White;
            this.label_Game.AutoSize = true;
            this.label_Game.Location = new Point(12, 84);
            this.label_Game.Name = "label_Game";
            this.label_Game.Size = new Size(96, 25);
            this.label_Game.TabIndex = 3;
            this.label_Game.Text = "Game: loading";
            this.label_Game.Visible = false;
            this.Controls.Add(this.label_Game);

            /*
            // label_Connection
            this.label_Connection.ForeColor = Color.White;
            this.label_Connection.AutoSize = true;
            this.label_Connection.Location = new Point(12, 109);
            this.label_Connection.Name = "label_Connection";
            this.label_Connection.Size = new Size(96, 25);
            this.label_Connection.TabIndex = 4;
            this.label_Connection.Text = "Connection: waiting for game";
            this.label_Connection.Visible = false;
            this.Controls.Add(this.label_Connection);
            */

            // label_Save
            this.label_Save.ForeColor = Color.White;
            this.label_Save.AutoSize = true;
            this.label_Save.Location = new Point(12, /*134*/ 109);
            this.label_Save.Name = "label_Save";
            this.label_Save.Size = new Size(96, 25);
            this.label_Save.TabIndex = /*5*/ 4;
            this.label_Save.Text = "Save: waiting for game";
            this.label_Save.Visible = false;
            this.Controls.Add(this.label_Save);

            // label_Help
            this.label_Help.ForeColor = Color.White;
            this.label_Help.AutoSize = true;
            this.label_Help.Location = new Point(12, /*159*/ 134);
            this.label_Help.Name = "label_Help";
            this.label_Help.Size = new Size(96, 25);
            this.label_Help.TabIndex = /*6*/ 5;
            this.label_Help.Text = "If you need help, you can ask in #setup-support on Discord.";
            this.label_Help.Visible = false;
            this.Controls.Add(this.label_Help);

            // button_Close_Menu
            this.button_Close_Menu.ForeColor = Color.White;
            this.button_Close_Menu.AutoSize = true;
            this.button_Close_Menu.Location = new Point(12, /*184*/ 159);
            this.button_Close_Menu.Name = "button_Close_Menu";
            this.button_Close_Menu.Size = new Size(96, 25);
            this.button_Close_Menu.TabIndex = /*7*/ 6;
            this.button_Close_Menu.Text = "Done";
            this.button_Close_Menu.Visible = false;
            this.button_Close_Menu.Click += new EventHandler((object sender, EventArgs e) => {
                if (this.WindowState == FormWindowState.Maximized) {
                    this.WindowState = FormWindowState.Normal;
                }
                this.FormBorderStyle = FormBorderStyle.FixedSingle;
                this.MaximizeBox = false;
                this.ClientSize = new Size(720, 896);
                this.label_Version.Visible = false;
                this.button_Update.Visible = false;
                this.label_Update.Visible = false;
                this.label_Game.Visible = false;
                //this.label_Connection.Visible = false;
                this.label_Save.Visible = false;
                this.label_Help.Visible = false;
                this.button_Close_Menu.Visible = false;
                foreach (PictureBox cell in this.cells) {
                    cell.Visible = true;
                }
            });
            this.Controls.Add(this.button_Close_Menu);

            ResumeLayout(true);
        }

        public override void Restart() {
            this.model.Dispose();
            /*
            if (this.stream != null) { this.stream.Disconnect().Dispose(); }
            this.stream = null;
            UpdateConnection(false, "Connection: waiting for game");
            */
            if (this.prevSave != null) { this.prevSave.Dispose(); }
            this.prevSave = null;
            UpdateSave(false, "Save: waiting for game");
            if ((APIs.GameInfo.GetGameInfo()?.Name ?? "Null") == "Null") {
                this.model = ModelState.FromSaveAndKnowledge(Native.save_default(), Native.knowledge_none());
                UpdateGame(false, "Not playing anything");
            } else {
                var rom_ident = APIs.Memory.ReadByteRange(0x20, 0x18, "ROM");
                if (!Enumerable.SequenceEqual(rom_ident.GetRange(0, 0x15), new List<byte>(Encoding.UTF8.GetBytes("THE LEGEND OF ZELDA \0")))) {
                    this.model = ModelState.FromSaveAndKnowledge(Native.save_default(), Native.knowledge_none());
                    UpdateGame(false, $"Game: Expected OoT or OoTR, found {APIs.GameInfo.GetGameInfo()?.Name ?? "Null"} ({string.Join<byte>(", ", rom_ident.GetRange(0, 0x15))})");
                } else {
                    var version = rom_ident.GetRange(0x15, 3);
                    this.isVanilla = Enumerable.SequenceEqual(version, new List<byte>(new byte[] { 0, 0, 0 }));
                    this.model = ModelState.FromSaveAndKnowledge(Native.save_default(), this.isVanilla ? Native.knowledge_vanilla() : Native.knowledge_none());
                    if (this.isVanilla) {
                        UpdateGame(true, "Playing OoT (vanilla)");
                    } else {
                        UpdateGame(true, $"Playing OoTR version {version[0]}.{version[1]}.{version[2]}");
                    }
                    /*
                    using (var stream_res = TcpStreamResult.Connect(IPAddress.IPv6Loopback)) { //TODO only connect manually
                        if (stream_res.IsOk()) {
                            if (this.stream != null) { this.stream.Disconnect().Dispose(); }
                            this.stream = stream_res.Unwrap();
                            UpdateConnection(true, "Connected");
                            if (this.isVanilla) {
                                using (var knowledge = Native.knowledge_vanilla()) { //TODO pull knowledge back out of this.model
                                    knowledge.Send(this.stream);
                                }
                            }
                        } else {
                            using (StringHandle err = stream_res.DebugErr()) {
                                UpdateConnection(false, $"Failed to connect: {err.AsString()}");
                            }
                        }
                    }
                    */
                }
            }
            UpdateCells();
        }

        public override void UpdateValues(ToolFormUpdateType type) {
            if (type != ToolFormUpdateType.PreFrame) { return; } //TODO setting to also enable auto-tracking during turbo (ToolFormUpdateType.FastPreFrame)?
            if ((APIs.GameInfo.GetGameInfo()?.Name ?? "Null") == "Null") { return; }
            bool changed = true;
            if (this.rawRam == null) {
                this.rawRam = new RawRam(APIs.Memory);
            } else {
                changed = this.rawRam.Update(APIs.Memory);
            }
            if (!changed) { return; }
            using (var ram_res = this.rawRam.ToRam()) {
                if (ram_res.IsOk()) {
                    var ram = ram_res.Unwrap();
                    if (prevRam != null && ram.Equals(prevRam)) { return; }
                    if (prevRam != null) { prevRam.Dispose(); }
                    prevRam = ram;
                } else {
                    UpdateSave(false, $"Failed to read game RAM: {ram_res.DebugErr().AsString()}");
                    return;
                }
            }
            UpdateSave(true, $"Save data ok, last checked {DateTime.Now}");
            this.model.SetRam(prevRam);
            UpdateCells();
            var save = prevRam.CloneSave();
            if (prevSave != null && save.Equals(prevSave)) { return; }
            if (prevSave == null) {
                /*
                if (this.stream != null) {
                    using (UnitResult unit_res = save.Send(this.stream)) {
                        if (!unit_res.IsOk()) {
                            if (this.stream != null) { this.stream.Dispose(); }
                            this.stream = null;
                            using (StringHandle err = unit_res.DebugErr()) {
                                UpdateConnection(false, $"Failed to send save data: {err.AsString()}");
                            }
                        } else {
                            UpdateConnection(true, $"Connected, initial save data sent {DateTime.Now}");
                        }
                    }
                }
                */
                prevSave = save;
            } else if (!save.Equals(prevSave)) {
                /*
                if (this.stream != null) {
                    using (SavesDiff diff = prevSave.Diff(save)) {
                        using (UnitResult unit_res = diff.Send(this.stream)) {
                            if (!unit_res.IsOk()) {
                                if (this.stream != null) { this.stream.Dispose(); }
                                this.stream = null;
                                using (StringHandle err = unit_res.DebugErr()) {
                                    UpdateConnection(false, $"Failed to send save data: {err.AsString()}");
                                }
                            } else {
                                UpdateConnection(true, $"Connected, save data last sent {DateTime.Now}");
                            }
                        }
                    }
                }
                */
                prevSave.Dispose();
                prevSave = save;
            } else {
                save.Dispose();
            }
        }

        private void UpdateCells() {
            for (byte i = 0; i < 52; i++) {
                using (TrackerCell cell = layout.Cell(i)) {
                    string new_img = cell.Image(this.model).AsString();
                    if (new_img == this.cellImages[i]) { continue; }
                    this.cellImages[i] = new_img;
                    var stream = typeof(MainForm).Assembly.GetManifestResourceStream($"Net.Fenhl.OotAutoTracker.Resources.{new_img}.png");
                    if (stream == null) { throw new Exception($"image stream for cell {i} ({new_img}) is null"); }
                    this.cells[i].Image = Image.FromStream(stream);
                }
            }
        }

        private void UpdateGame(bool ok, String msg) {
            label_Game.Text = msg;
            this.gameOk = ok;
            UpdateHelpLabel();
        }

        /*
        private void UpdateConnection(bool ok, String msg) {
            label_Connection.Text = msg;
            this.connectionOk = ok;
            UpdateHelpLabel();
        }
        */

        private void UpdateSave(bool ok, String msg) {
            label_Save.Text = msg;
            this.saveOk = ok;
            UpdateHelpLabel();
        }

        private void UpdateHelpLabel() {
            if (this.gameOk /*&& this.connectionOk*/ && this.saveOk) {
                label_Help.Text = "";
            } else {
                label_Help.Text = "If you need help, you can ask in #setup-support on Discord.";
            }
        }
    }
}
