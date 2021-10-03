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
        [DllImport("oottracker")] internal static extern StringHandle version_string();
        [DllImport("oottracker")] internal static extern TrackerLayoutHandle layout_default();
        [DllImport("oottracker")] internal static extern void layout_free(IntPtr layout);
        [DllImport("oottracker")] internal static extern TrackerCellHandle layout_cell(TrackerLayoutHandle layout, byte idx);
        [DllImport("oottracker")] internal static extern void cell_free(IntPtr layout);
        [DllImport("oottracker")] internal static extern StringHandle cell_image(ModelStateHandle model, TrackerCellHandle cell);
        [DllImport("oottracker")] internal static extern TcpStreamResultHandle connect_ipv4(byte[] addr);
        [DllImport("oottracker")] internal static extern TcpStreamResultHandle connect_ipv6(byte[] addr);
        [DllImport("oottracker")] internal static extern void tcp_stream_result_free(IntPtr tcp_stream_res);
        [DllImport("oottracker")] internal static extern bool tcp_stream_result_is_ok(TcpStreamResultHandle tcp_stream_res);
        [DllImport("oottracker")] internal static extern TcpStreamHandle tcp_stream_result_unwrap(IntPtr tcp_stream_res);
        [DllImport("oottracker")] internal static extern void tcp_stream_free(IntPtr tcp_stream);
        [DllImport("oottracker")] internal static extern StringHandle tcp_stream_result_debug_err(IntPtr tcp_stream_res);
        [DllImport("oottracker")] internal static extern void string_free(IntPtr s);
        [DllImport("oottracker")] internal static extern UnitResultHandle tcp_stream_disconnect(IntPtr tcp_stream);
        [DllImport("oottracker")] internal static extern void unit_result_free(IntPtr unit_res);
        [DllImport("oottracker")] internal static extern bool unit_result_is_ok(UnitResultHandle unit_res);
        [DllImport("oottracker")] internal static extern StringHandle unit_result_debug_err(IntPtr unit_res);
        [DllImport("oottracker")] internal static extern SaveResultHandle save_from_save_data(byte[] start);
        [DllImport("oottracker")] internal static extern void save_result_free(IntPtr save_res);
        [DllImport("oottracker")] internal static extern bool save_result_is_ok(SaveResultHandle save_res);
        [DllImport("oottracker")] internal static extern SaveHandle save_result_unwrap(IntPtr save_res);
        [DllImport("oottracker")] internal static extern SaveHandle save_default();
        [DllImport("oottracker")] internal static extern void save_free(IntPtr save);
        [DllImport("oottracker")] internal static extern StringHandle save_debug(SaveHandle save);
        [DllImport("oottracker")] internal static extern StringHandle save_result_debug_err(IntPtr save_res);
        [DllImport("oottracker")] internal static extern UnitResultHandle save_send(TcpStreamHandle tcp_stream, SaveHandle save);
        [DllImport("oottracker")] internal static extern bool saves_equal(SaveHandle save1, SaveHandle save2);
        [DllImport("oottracker")] internal static extern SavesDiffHandle saves_diff(SaveHandle old_save, SaveHandle new_save);
        [DllImport("oottracker")] internal static extern void saves_diff_free(IntPtr diff);
        [DllImport("oottracker")] internal static extern UnitResultHandle saves_diff_send(TcpStreamHandle tcp_stream, IntPtr diff);
        [DllImport("oottracker")] internal static extern KnowledgeHandle knowledge_none();
        [DllImport("oottracker")] internal static extern KnowledgeHandle knowledge_vanilla();
        [DllImport("oottracker")] internal static extern void knowledge_free(IntPtr knowledge);
        [DllImport("oottracker")] internal static extern UnitResultHandle knowledge_send(TcpStreamHandle tcp_stream, KnowledgeHandle knowledge);
        [DllImport("oottracker")] internal static extern ModelStateHandle model_new(IntPtr save, IntPtr knowledge);
        [DllImport("oottracker")] internal static extern void model_free(IntPtr model);
        [DllImport("oottracker")] internal static extern byte ram_num_ranges();
        [DllImport("oottracker")] internal static extern IntPtr ram_ranges();
        [DllImport("oottracker")] internal static extern RamResultHandle ram_from_ranges(IntPtr[] ranges);
        [DllImport("oottracker")] internal static extern void ram_result_free(IntPtr ram_res);
        [DllImport("oottracker")] internal static extern bool ram_result_is_ok(RamResultHandle ram_res);
        [DllImport("oottracker")] internal static extern RamHandle ram_result_unwrap(IntPtr ram_res);
        [DllImport("oottracker")] internal static extern StringHandle ram_result_debug_err(IntPtr ram_res);
        [DllImport("oottracker")] internal static extern void ram_free(IntPtr ram);
        [DllImport("oottracker")] internal static extern bool ram_equal(RamHandle ram1, RamHandle ram2);
        [DllImport("oottracker")] internal static extern void model_set_ram(ModelStateHandle model, RamHandle ram);
        [DllImport("oottracker")] internal static extern SaveHandle ram_clone_save(RamHandle ram);
    }

    internal class TrackerLayoutHandle : SafeHandle {
        internal TrackerLayoutHandle() : base(IntPtr.Zero, true) { }

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.layout_free(handle);
            }
            return true;
        }
    }

    class TrackerLayout : IDisposable {
        internal TrackerLayoutHandle layout;

        internal TrackerLayout() {
            layout = Native.layout_default();
        }

        public void Dispose() {
            layout.Dispose();
        }
    }

    internal class TrackerCellHandle : SafeHandle {
        internal TrackerCellHandle() : base(IntPtr.Zero, true) { }

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.cell_free(handle);
            }
            return true;
        }
    }

    class TrackerCell : IDisposable {
        internal TrackerCellHandle cell;

        internal TrackerCell(TrackerLayout layout, byte idx) {
            cell = Native.layout_cell(layout.layout, idx);
        }

        public void Dispose() {
            cell.Dispose();
        }

        public StringHandle Image(ModelState model) => Native.cell_image(model.model, cell);
    }

    internal class TcpStreamResultHandle : SafeHandle {
        internal TcpStreamResultHandle() : base(IntPtr.Zero, true) { }

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.tcp_stream_result_free(handle);
            }
            return true;
        }

        internal TcpStreamHandle Unwrap() {
            var tcp_stream = Native.tcp_stream_result_unwrap(handle);
            this.handle = IntPtr.Zero; // tcp_stream_result_unwrap takes ownership
            return tcp_stream;
        }

        internal StringHandle DebugErr() {
            var err = Native.tcp_stream_result_debug_err(handle);
            this.handle = IntPtr.Zero; // tcp_stream_result_debug_err takes ownership
            return err;
        }
    }

    internal class TcpStreamResult : IDisposable {
        internal TcpStreamResultHandle tcp_stream_res;

        internal TcpStreamResult(IPAddress addr) {
            tcp_stream_res = addr.AddressFamily switch {
                AddressFamily.InterNetwork => Native.connect_ipv4(addr.GetAddressBytes().ToArray()),
                AddressFamily.InterNetworkV6 => Native.connect_ipv6(addr.GetAddressBytes().ToArray()),
            };
        }

        public void Dispose() {
            tcp_stream_res.Dispose();
        }

        internal bool IsOk() => Native.tcp_stream_result_is_ok(tcp_stream_res);
        internal TcpStreamHandle Unwrap() => tcp_stream_res.Unwrap();
        internal StringHandle DebugErr() => tcp_stream_res.DebugErr();
    }

    internal class TcpStreamHandle : SafeHandle {
        internal TcpStreamHandle() : base(IntPtr.Zero, true) { }

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.tcp_stream_free(handle);
            }
            return true;
        }

        internal UnitResultHandle Disconnect() {
            var unit_res = Native.tcp_stream_disconnect(handle);
            this.handle = IntPtr.Zero; // tcp_stream_disconnect takes ownership
            return unit_res;
        }
    }

    class TcpStream : IDisposable {
        internal TcpStreamHandle tcp_stream;

        internal TcpStream(TcpStreamResult tcp_stream_res) {
            tcp_stream = tcp_stream_res.Unwrap();
        }

        public void Dispose() {
            tcp_stream.Dispose();
        }

        internal UnitResult Disconnect() {
            return new UnitResult(tcp_stream.Disconnect());
        }
    }

    internal class UnitResultHandle : SafeHandle {
        internal UnitResultHandle() : base(IntPtr.Zero, true) { }

        public override bool IsInvalid {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.unit_result_free(handle);
            }
            return true;
        }

        internal StringHandle DebugErr() {
            var err = Native.unit_result_debug_err(handle);
            this.handle = IntPtr.Zero; // unit_result_debug_err takes ownership
            return err;
        }
    }

    internal class UnitResult : IDisposable
    {
        internal UnitResultHandle unit_res;

        internal UnitResult(UnitResultHandle unit_res) {
            this.unit_res = unit_res;
        }

        public void Dispose() {
            unit_res.Dispose();
        }

        internal bool IsOk() => Native.unit_result_is_ok(unit_res);
        internal StringHandle DebugErr() => unit_res.DebugErr();
    }

    internal class SaveResultHandle : SafeHandle
    {
        internal SaveResultHandle() : base(IntPtr.Zero, true) { }

        public override bool IsInvalid
        {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.save_result_free(handle);
            }
            return true;
        }

        internal SaveHandle Unwrap() {
            var save = Native.save_result_unwrap(handle);
            this.handle = IntPtr.Zero; // save_result_unwrap takes ownership
            return save;
        }

        internal StringHandle DebugErr() {
            var err = Native.save_result_debug_err(handle);
            this.handle = IntPtr.Zero; // save_result_debug_err takes ownership
            return err;
        }
    }
    class SaveResult : IDisposable
    {
        internal SaveResultHandle save_res;

        internal SaveResult(List<byte> save_data) {
            save_res = Native.save_from_save_data(save_data.ToArray());
        }

        public void Dispose() {
            save_res.Dispose();
        }

        internal bool IsOk() => Native.save_result_is_ok(save_res);
        internal SaveHandle Unwrap() => save_res.Unwrap();
        internal StringHandle DebugErr() => save_res.DebugErr();
    }

    internal class StringHandle : SafeHandle
    {
        internal StringHandle() : base(IntPtr.Zero, true) {}

        public override bool IsInvalid
        {
            get { return this.handle == IntPtr.Zero; }
        }

        public string AsString() {
            int len = 0;
            while (Marshal.ReadByte(handle, len) != 0) { ++len; }
            byte[] buffer = new byte[len];
            Marshal.Copy(handle, buffer, 0, buffer.Length);
            return Encoding.UTF8.GetString(buffer);
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.string_free(handle);
            }
            return true;
        }
    }

    internal class SaveHandle : SafeHandle
    {
        internal SaveHandle() : base(IntPtr.Zero, true) {}

        public override bool IsInvalid
        {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.save_free(handle);
            }
            return true;
        }

        public IntPtr Move() {
            var ptr = this.handle;
            this.handle = IntPtr.Zero;
            return ptr;
        }
    }

    class Save : IDisposable
    {
        internal SaveHandle save;

        internal Save() {
            save = Native.save_default();
        }

        internal Save(SaveResult save_res) {
            save = save_res.Unwrap();
        }

        internal Save(Ram ram) {
            save = Native.ram_clone_save(ram.ram);
        }

        internal bool Equals(Save other) {
            return Native.saves_equal(save, other.save);
        }

        internal SavesDiff Diff(Save other) {
            return new SavesDiff(save, other.save);
        }

        internal UnitResult Send(TcpStream tcp_stream) {
            return new UnitResult(Native.save_send(tcp_stream.tcp_stream, save));
        }

        internal StringHandle Debug() {
            return Native.save_debug(save);
        }

        public void Dispose() {
            save.Dispose();
        }
    }

    internal class SavesDiffHandle : SafeHandle
    {
        internal SavesDiffHandle() : base(IntPtr.Zero, true) { }

        public override bool IsInvalid
        {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.saves_diff_free(handle);
            }
            return true;
        }

        internal UnitResultHandle Send(TcpStreamHandle tcp_stream) {
            var unit_res = Native.saves_diff_send(tcp_stream, handle);
            this.handle = IntPtr.Zero; // saves_diff_send takes ownership
            return unit_res;
        }
    }

    class SavesDiff : IDisposable
    {
        private SavesDiffHandle diff;

        internal SavesDiff(SaveHandle old_save, SaveHandle new_save) {
            diff = Native.saves_diff(old_save, new_save);
        }

        public void Dispose() {
            diff.Dispose();
        }

        internal UnitResult Send(TcpStream tcp_stream) {
            return new UnitResult(diff.Send(tcp_stream.tcp_stream));
        }
    }

    internal class KnowledgeHandle : SafeHandle
    {
        internal KnowledgeHandle() : base(IntPtr.Zero, true) { }

        public override bool IsInvalid
        {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.knowledge_free(handle);
            }
            return true;
        }

        public IntPtr Move() {
            var ptr = this.handle;
            this.handle = IntPtr.Zero;
            return ptr;
        }
    }

    class Knowledge : IDisposable
    {
        internal KnowledgeHandle knowledge;

        internal Knowledge(bool isVanilla) {
            if (isVanilla) {
                knowledge = Native.knowledge_vanilla();
            } else {
                knowledge = Native.knowledge_none();
            }
        }

        internal UnitResult Send(TcpStream tcp_stream) {
            return new UnitResult(Native.knowledge_send(tcp_stream.tcp_stream, knowledge));
        }

        public void Dispose() {
            knowledge.Dispose();
        }
    }

    internal class ModelStateHandle : SafeHandle
    {
        internal ModelStateHandle() : base(IntPtr.Zero, true) { }

        public override bool IsInvalid
        {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.model_free(handle);
            }
            return true;
        }
    }

    class ModelState : IDisposable
    {
        internal ModelStateHandle model;

        internal ModelState(Save save, Knowledge knowledge) {
            var save_ptr = save.save.Move();
            var knowledge_ptr = knowledge.knowledge.Move();
            model = Native.model_new(save_ptr, knowledge_ptr);
        }

        public void Dispose() {
            model.Dispose();
        }

        public void SetRam(Ram ram) {
            Native.model_set_ram(model, ram.ram);
        }
    }

    internal class RamResultHandle : SafeHandle
    {
        internal RamResultHandle() : base(IntPtr.Zero, true) { }

        public override bool IsInvalid
        {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.ram_result_free(handle);
            }
            return true;
        }

        internal RamHandle Unwrap() {
            var ram = Native.ram_result_unwrap(handle);
            this.handle = IntPtr.Zero; // ram_result_unwrap takes ownership
            return ram;
        }

        internal StringHandle DebugErr() {
            var err = Native.ram_result_debug_err(handle);
            this.handle = IntPtr.Zero; // ram_result_debug_err takes ownership
            return err;
        }
    }
    class RamResult : IDisposable
    {
        internal RamResultHandle ram_res;

        internal RamResult(RawRam rawRam) {
            IntPtr[] range_data = new IntPtr[rawRam.num_ranges];
            for (byte i = 0; i < rawRam.num_ranges; i++) {
                range_data[i] = Marshal.AllocHGlobal(rawRam.ranges[2 * i + 1]);
                Marshal.Copy(rawRam.range_data[i], 0, range_data[i], rawRam.ranges[2 * i + 1]);
            }
            ram_res = Native.ram_from_ranges(range_data);
            for (byte i = 0; i < rawRam.num_ranges; i++) {
                Marshal.FreeHGlobal(range_data[i]);
            }
        }

        public void Dispose() {
            ram_res.Dispose();
        }

        internal bool IsOk() => Native.ram_result_is_ok(ram_res);
        internal RamHandle Unwrap() => ram_res.Unwrap();
        internal StringHandle DebugErr() => ram_res.DebugErr();
    }

    internal class RamHandle : SafeHandle
    {
        internal RamHandle() : base(IntPtr.Zero, true) { }

        public override bool IsInvalid
        {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle() {
            if (!this.IsInvalid) {
                Native.ram_free(handle);
            }
            return true;
        }

        public IntPtr Move() {
            var ptr = this.handle;
            this.handle = IntPtr.Zero;
            return ptr;
        }
    }

    class Ram : IDisposable
    {
        internal RamHandle ram;

        internal Ram(RamResult ram_res) {
            ram = ram_res.Unwrap();
        }

        public void Dispose() {
            ram.Dispose();
        }

        internal bool Equals(Ram other) {
            return Native.ram_equal(ram, other.ram);
        }
    }

    class RawRam
    {
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
    }

    [ExternalTool("OoT autotracker", Description = "An auto-tracking plugin for Fenhl's OoT tracker")]
    [ExternalToolEmbeddedIcon("Net.Fenhl.OotAutoTracker.Resources.icon.ico")]
    public sealed class MainForm : ToolFormBase, IExternalToolForm
    {
        private PictureBox[] cells = new PictureBox[52];
        private Label label_Version;
        private Label label_Game;
        private Label label_Connection;
        private Label label_Save;
        private Label label_Help;
        private Button button_Close_Menu;

        public ApiContainer? _apiContainer { get; set; }

        private ApiContainer APIs => _apiContainer ?? throw new NullReferenceException();

        private bool isVanilla;
        private TcpStream? stream;
        private RawRam? rawRam;
        private Ram? prevRam;
        private List<byte> prevSaveData = new List<byte>();
        private Save? prevSave;
        private ModelState model = new ModelState(new Save(), new Knowledge(false));
        private TrackerLayout layout = new TrackerLayout();
        private string[] cellImages = new string[52];

        private bool gameOk = false;
        private bool connectionOk = false;
        private bool saveOk = false;

        public MainForm() {
            InitializeComponent();
            ClientSize = new Size(720, 896);
            Icon = new Icon(typeof(MainForm).Assembly.GetManifestResourceStream("Net.Fenhl.OotAutoTracker.Resources.icon.ico"));
            SuspendLayout();
            ResumeLayout();
        }

        public override bool BlocksInputWhenFocused { get; } = false;
        protected override string WindowTitleStatic => "OoT autotracker";

        public bool AskSaveChanges() => true;

        public void Restart() {
            this.model.Dispose();
            if (this.stream != null) { this.stream.Disconnect().Dispose(); }
            this.stream = null;
            UpdateConnection(false, "Connection: waiting for game");
            if (this.prevSave != null) { this.prevSave.Dispose(); }
            this.prevSave = null;
            UpdateSave(false, "Save: waiting for game");
            if (APIs.GameInfo.GetRomName() == "Null") {
                this.model = new ModelState(new Save(), new Knowledge(false));
                UpdateGame(false, "Not playing anything");
            } else {
                var rom_ident = APIs.Memory.ReadByteRange(0x20, 0x18, "ROM");
                if (!Enumerable.SequenceEqual(rom_ident.GetRange(0, 0x15), new List<byte>(Encoding.UTF8.GetBytes("THE LEGEND OF ZELDA \0")))) {
                    this.model = new ModelState(new Save(), new Knowledge(false));
                    UpdateGame(false, $"Game: Expected OoT or OoTR, found {APIs.GameInfo.GetRomName()} ({string.Join<byte>(", ", rom_ident.GetRange(0, 0x15))})");
                } else {
                    var version = rom_ident.GetRange(0x15, 3);
                    this.isVanilla = Enumerable.SequenceEqual(version, new List<byte>(new byte[] { 0, 0, 0 }));
                    this.model = new ModelState(new Save(), new Knowledge(this.isVanilla));
                    if (this.isVanilla) {
                        UpdateGame(true, "Playing OoT (vanilla)");
                    } else {
                        UpdateGame(true, $"Playing OoTR version {version[0]}.{version[1]}.{version[2]}");
                    }
                    using (var stream_res = new TcpStreamResult(IPAddress.IPv6Loopback)) { //TODO only connect manually
                        if (stream_res.IsOk()) {
                            if (this.stream != null) { this.stream.Disconnect().Dispose(); }
                            this.stream = new TcpStream(stream_res);
                            UpdateConnection(true, "Connected");
                            if (this.isVanilla) {
                                using (var knowledge = new Knowledge(true)) { //TODO pull knowledge back out of this.model
                                    knowledge.Send(this.stream);
                                }
                            }
                        } else {
                            using (StringHandle err = stream_res.DebugErr()) {
                                UpdateConnection(false, $"Failed to connect: {err.AsString()}");
                            }
                        }
                    }
                }
            }
            UpdateCells();
        }

        public void UpdateValues(ToolFormUpdateType type) {
            if (type != ToolFormUpdateType.PreFrame) { return; } //TODO setting to also enable auto-tracking during turbo (ToolFormUpdateType.FastPreFrame)?
            if (APIs.GameInfo.GetRomName() == "Null") { return; }
            bool changed = true;
            if (this.rawRam == null) {
                this.rawRam = new RawRam(APIs.Memory);
            } else {
                changed = this.rawRam.Update(APIs.Memory);
            }
            if (!changed) { return; }
            using (var ram_res = new RamResult(this.rawRam)) {
                if (ram_res.IsOk()) {
                    var ram = new Ram(ram_res);
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
            var save = new Save(prevRam);
            if (prevSave != null && save.Equals(prevSave)) { return; }
            if (prevSave == null) {
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
                prevSave = save;
            } else if (!save.Equals(prevSave)) {
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
                prevSave.Dispose();
                prevSave = save;
            } else {
                save.Dispose();
            }
        }

        private void UpdateCells() {
            for (byte i = 0; i < 52; i++) {
                using (TrackerCell cell = new TrackerCell(this.layout, i)) {
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

        private void UpdateConnection(bool ok, String msg) {
            label_Connection.Text = msg;
            this.connectionOk = ok;
            UpdateHelpLabel();
        }

        private void UpdateSave(bool ok, String msg) {
            label_Save.Text = msg;
            this.saveOk = ok;
            UpdateHelpLabel();
        }

        private void UpdateHelpLabel() {
            if (this.gameOk && this.connectionOk && this.saveOk) {
                label_Help.Text = "You can now minimize this window. To stop auto-tracking, close this window.";
            } else {
                label_Help.Text = "If you need help, you can ask in #setup-support on Discord.";
            }
        }

        private void InitializeComponent() {
            this.label_Version = new Label();
            this.label_Game = new Label();
            this.label_Connection = new Label();
            this.label_Save = new Label();
            this.label_Help = new Label();
            this.button_Close_Menu = new Button();
            this.SuspendLayout();
            //
            // cells
            //
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
                            this.label_Game.Visible = true;
                            this.label_Connection.Visible = true;
                            this.label_Save.Visible = true;
                            this.label_Help.Visible = true;
                            this.button_Close_Menu.Visible = true;
                            foreach (PictureBox cell in this.cells) {
                                cell.Visible = false;
                            }
                        }
                    });
                }
                this.Controls.Add(cell);
            }
            this.UpdateCells();
            //
            // label_Version
            //
            this.label_Version.ForeColor = Color.White;
            this.label_Version.AutoSize = true;
            this.label_Version.Location = new Point(12, 9);
            this.label_Version.Name = "label_Version";
            this.label_Version.Size = new Size(96, 25);
            this.label_Version.TabIndex = 0;
            this.label_Version.Text = $"OoT autotracker version {Native.version_string().AsString()}";
            this.label_Version.Visible = false;
            //
            // label_Game
            //
            this.label_Game.ForeColor = Color.White;
            this.label_Game.AutoSize = true;
            this.label_Game.Location = new Point(12, 34);
            this.label_Game.Name = "label_Game";
            this.label_Game.Size = new Size(96, 25);
            this.label_Game.TabIndex = 1;
            this.label_Game.Text = "Game: loading";
            this.label_Game.Visible = false;
            //
            // label_Connection
            //
            this.label_Connection.ForeColor = Color.White;
            this.label_Connection.AutoSize = true;
            this.label_Connection.Location = new Point(12, 59);
            this.label_Connection.Name = "label_Connection";
            this.label_Connection.Size = new Size(96, 25);
            this.label_Connection.TabIndex = 2;
            this.label_Connection.Text = "Connection: waiting for game";
            this.label_Connection.Visible = false;
            //
            // label_Save
            //
            this.label_Save.ForeColor = Color.White;
            this.label_Save.AutoSize = true;
            this.label_Save.Location = new Point(12, 84);
            this.label_Save.Name = "label_Save";
            this.label_Save.Size = new Size(96, 25);
            this.label_Save.TabIndex = 3;
            this.label_Save.Text = "Save: waiting for game";
            this.label_Save.Visible = false;
            //
            // label_Help
            //
            this.label_Help.ForeColor = Color.White;
            this.label_Help.AutoSize = true;
            this.label_Help.Location = new Point(12, 109);
            this.label_Help.Name = "label_Help";
            this.label_Help.Size = new Size(96, 25);
            this.label_Help.TabIndex = 4;
            this.label_Help.Text = "If you need help, you can ask in #setup-support on Discord.";
            this.label_Help.Visible = false;
            //
            // button_Close_Menu
            //
            this.button_Close_Menu.ForeColor = Color.White;
            this.button_Close_Menu.AutoSize = true;
            this.button_Close_Menu.Location = new Point(12, 134);
            this.button_Close_Menu.Name = "button_Close_Menu";
            this.button_Close_Menu.Size = new Size(96, 25);
            this.button_Close_Menu.TabIndex = 5;
            this.button_Close_Menu.Text = "Done";
            this.button_Close_Menu.Visible = false;
            this.button_Close_Menu.Click += new EventHandler((object sender, EventArgs e) => {
                this.ClientSize = new Size(720, 896);
                this.label_Version.Visible = false;
                this.label_Game.Visible = false;
                this.label_Connection.Visible = false;
                this.label_Save.Visible = false;
                this.label_Help.Visible = false;
                this.button_Close_Menu.Visible = false;
                foreach (PictureBox cell in this.cells) {
                    cell.Visible = true;
                }
            });
            //
            // MainForm
            //
            this.BackColor = Color.Black;
            this.AutoScaleMode = AutoScaleMode.Dpi;
            this.Controls.Add(this.label_Version);
            this.Controls.Add(this.label_Game);
            this.Controls.Add(this.label_Connection);
            this.Controls.Add(this.label_Save);
            this.Controls.Add(this.label_Help);
            this.Controls.Add(this.button_Close_Menu);
            this.Name = "MainForm";
            this.ResumeLayout(false);
            this.PerformLayout();
        }
    }
}
