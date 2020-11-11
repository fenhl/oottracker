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

namespace Net.Fenhl.OotAutoTracker
{
    internal class Native
    {
        [DllImport("oottracker")]
        internal static extern StringHandle version_string();
        [DllImport("oottracker")]
        internal static extern TcpStreamResultHandle connect_ipv4(byte[] addr);
        [DllImport("oottracker")]
        internal static extern TcpStreamResultHandle connect_ipv6(byte[] addr);
        [DllImport("oottracker")]
        internal static extern void tcp_stream_result_free(IntPtr tcp_stream_res);
        [DllImport("oottracker")]
        internal static extern bool tcp_stream_result_is_ok(TcpStreamResultHandle tcp_stream_res);
        [DllImport("oottracker")]
        internal static extern TcpStreamHandle tcp_stream_result_unwrap(IntPtr tcp_stream_res);
        [DllImport("oottracker")]
        internal static extern void tcp_stream_free(IntPtr tcp_stream);
        [DllImport("oottracker")]
        internal static extern StringHandle tcp_stream_result_debug_err(IntPtr tcp_stream_res);
        [DllImport("oottracker")]
        internal static extern void string_free(IntPtr s);
        [DllImport("oottracker")]
        internal static extern IoResultHandle tcp_stream_disconnect(IntPtr tcp_stream);
        [DllImport("oottracker")]
        internal static extern void io_result_free(IntPtr io_res);
        [DllImport("oottracker")]
        internal static extern bool io_result_is_ok(IoResultHandle io_res);
        [DllImport("oottracker")]
        internal static extern StringHandle io_result_debug_err(IntPtr io_res);
        [DllImport("oottracker")]
        internal static extern SaveResultHandle save_from_save_data(byte[] start);
        [DllImport("oottracker")]
        internal static extern void save_result_free(IntPtr save_res);
        [DllImport("oottracker")]
        internal static extern bool save_result_is_ok(SaveResultHandle save_res);
        [DllImport("oottracker")]
        internal static extern SaveHandle save_result_unwrap(IntPtr save_res);
        [DllImport("oottracker")]
        internal static extern void save_free(IntPtr save);
        [DllImport("oottracker")]
        internal static extern StringHandle save_debug(SaveHandle save);
        [DllImport("oottracker")]
        internal static extern StringHandle save_result_debug_err(IntPtr save_res);
        [DllImport("oottracker")]
        internal static extern IoResultHandle save_send(TcpStreamHandle tcp_stream, SaveHandle save);
        [DllImport("oottracker")]
        internal static extern bool saves_equal(SaveHandle save1, SaveHandle save2);
        [DllImport("oottracker")]
        internal static extern SavesDiffHandle saves_diff(SaveHandle old_save, SaveHandle new_save);
        [DllImport("oottracker")]
        internal static extern void saves_diff_free(IntPtr diff);
        [DllImport("oottracker")]
        internal static extern IoResultHandle saves_diff_send(TcpStreamHandle tcp_stream, IntPtr diff);
        [DllImport("oottracker")]
        internal static extern KnowledgeHandle knowledge_none();
        [DllImport("oottracker")]
        internal static extern KnowledgeHandle knowledge_vanilla();
        [DllImport("oottracker")]
        internal static extern void knowledge_free(IntPtr knowledge);
        [DllImport("oottracker")]
        internal static extern IoResultHandle knowledge_send(TcpStreamHandle tcp_stream, KnowledgeHandle knowledge);
    }

    internal class TcpStreamResultHandle : SafeHandle
    {
        internal TcpStreamResultHandle() : base(IntPtr.Zero, true) { }

        public override bool IsInvalid
        {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle()
        {
            if (!this.IsInvalid)
            {
                Native.tcp_stream_result_free(handle);
            }
            return true;
        }

        internal TcpStreamHandle Unwrap()
        {
            var tcp_stream = Native.tcp_stream_result_unwrap(handle);
            this.handle = IntPtr.Zero; // tcp_stream_result_unwrap takes ownership
            return tcp_stream;
        }

        internal StringHandle DebugErr()
        {
            var err = Native.tcp_stream_result_debug_err(handle);
            this.handle = IntPtr.Zero; // tcp_stream_result_debug_err takes ownership
            return err;
        }
    }

    internal class TcpStreamResult : IDisposable
    {
        internal TcpStreamResultHandle tcp_stream_res;

        internal TcpStreamResult(IPAddress addr)
        {
            tcp_stream_res = addr.AddressFamily switch
            {
                AddressFamily.InterNetwork => Native.connect_ipv4(addr.GetAddressBytes().ToArray()),
                AddressFamily.InterNetworkV6 => Native.connect_ipv6(addr.GetAddressBytes().ToArray()),
            };
        }

        public void Dispose()
        {
            tcp_stream_res.Dispose();
        }

        internal bool IsOk() => Native.tcp_stream_result_is_ok(tcp_stream_res);
        internal TcpStreamHandle Unwrap() => tcp_stream_res.Unwrap();
        internal StringHandle DebugErr() => tcp_stream_res.DebugErr();
    }

    internal class TcpStreamHandle : SafeHandle
    {
        internal TcpStreamHandle() : base(IntPtr.Zero, true) { }

        public override bool IsInvalid
        {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle()
        {
            if (!this.IsInvalid)
            {
                Native.tcp_stream_free(handle);
            }
            return true;
        }

        internal IoResultHandle Disconnect()
        {
            var io_res = Native.tcp_stream_disconnect(handle);
            this.handle = IntPtr.Zero; // tcp_stream_disconnect takes ownership
            return io_res;
        }
    }

    class TcpStream : IDisposable
    {
        internal TcpStreamHandle tcp_stream;

        internal TcpStream(TcpStreamResult tcp_stream_res)
        {
            tcp_stream = tcp_stream_res.Unwrap();
        }

        public void Dispose()
        {
            tcp_stream.Dispose();
        }

        internal IoResult Disconnect()
        {
            return new IoResult(tcp_stream.Disconnect());
        }
    }

    internal class IoResultHandle : SafeHandle
    {
        internal IoResultHandle() : base(IntPtr.Zero, true) { }

        public override bool IsInvalid
        {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle()
        {
            if (!this.IsInvalid)
            {
                Native.io_result_free(handle);
            }
            return true;
        }

        internal StringHandle DebugErr()
        {
            var err = Native.io_result_debug_err(handle);
            this.handle = IntPtr.Zero; // io_result_debug_err takes ownership
            return err;
        }
    }

    internal class IoResult : IDisposable
    {
        internal IoResultHandle io_res;

        internal IoResult(IoResultHandle io_res)
        {
            this.io_res = io_res;
        }

        public void Dispose()
        {
            io_res.Dispose();
        }

        internal bool IsOk() => Native.io_result_is_ok(io_res);
        internal StringHandle DebugErr() => io_res.DebugErr();
    }

    internal class SaveResultHandle : SafeHandle
    {
        internal SaveResultHandle() : base(IntPtr.Zero, true) { }

        public override bool IsInvalid
        {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle()
        {
            if (!this.IsInvalid)
            {
                Native.save_result_free(handle);
            }
            return true;
        }

        internal SaveHandle Unwrap()
        {
            var save = Native.save_result_unwrap(handle);
            this.handle = IntPtr.Zero; // state_result_unwrap takes ownership
            return save;
        }

        internal StringHandle DebugErr()
        {
            var err = Native.save_result_debug_err(handle);
            this.handle = IntPtr.Zero; // state_result_debug_err takes ownership
            return err;
        }
    }
    class SaveResult : IDisposable
    {
        internal SaveResultHandle save_res;

        internal SaveResult(List<byte> save_data)
        {
            save_res = Native.save_from_save_data(save_data.ToArray());
        }

        public void Dispose()
        {
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

        public string AsString()
        {
            int len = 0;
            while (Marshal.ReadByte(handle, len) != 0) { ++len; }
            byte[] buffer = new byte[len];
            Marshal.Copy(handle, buffer, 0, buffer.Length);
            return Encoding.UTF8.GetString(buffer);
        }

        protected override bool ReleaseHandle()
        {
            if (!this.IsInvalid)
            {
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

        protected override bool ReleaseHandle()
        {
            if (!this.IsInvalid)
            {
                Native.save_free(handle);
            }
            return true;
        }
    }

    class Save : IDisposable
    {
        private SaveHandle save;

        internal Save(SaveResult save_res)
        {
            save = save_res.Unwrap();
        }

        internal bool Equals(Save other)
        {
            return Native.saves_equal(save, other.save);
        }

        internal SavesDiff Diff(Save other)
        {
            return new SavesDiff(save, other.save);
        }

        internal IoResult Send(TcpStream tcp_stream)
        {
            return new IoResult(Native.save_send(tcp_stream.tcp_stream, save));
        }

        internal StringHandle Debug()
        {
            return Native.save_debug(save);
        }

        public void Dispose()
        {
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

        protected override bool ReleaseHandle()
        {
            if (!this.IsInvalid)
            {
                Native.saves_diff_free(handle);
            }
            return true;
        }

        internal IoResultHandle Send(TcpStreamHandle tcp_stream)
        {
            var io_res = Native.saves_diff_send(tcp_stream, handle);
            this.handle = IntPtr.Zero; // saves_diff_send takes ownership
            return io_res;
        }
    }

    class SavesDiff : IDisposable
    {
        private SavesDiffHandle diff;

        internal SavesDiff(SaveHandle old_save, SaveHandle new_save)
        {
            diff = Native.saves_diff(old_save, new_save);
        }

        public void Dispose()
        {
            diff.Dispose();
        }

        internal IoResult Send(TcpStream tcp_stream)
        {
            return new IoResult(diff.Send(tcp_stream.tcp_stream));
        }
    }

    internal class KnowledgeHandle : SafeHandle
    {
        internal KnowledgeHandle() : base(IntPtr.Zero, true) { }

        public override bool IsInvalid
        {
            get { return this.handle == IntPtr.Zero; }
        }

        protected override bool ReleaseHandle()
        {
            if (!this.IsInvalid)
            {
                Native.knowledge_free(handle);
            }
            return true;
        }
    }

    class Knowledge : IDisposable
    {
        private KnowledgeHandle knowledge;

        internal Knowledge(bool isVanilla)
        {
            if (isVanilla)
            {
                knowledge = Native.knowledge_vanilla();
            }
            else
            {
                knowledge = Native.knowledge_none();
            }
        }

        internal IoResult Send(TcpStream tcp_stream)
        {
            return new IoResult(Native.knowledge_send(tcp_stream.tcp_stream, knowledge));
        }

        public void Dispose()
        {
            knowledge.Dispose();
        }
    }

    [ExternalTool("OoT autotracker", Description = "An auto-tracking plugin for Fenhl's OoT tracker")]
	public sealed class MainForm : Form, IExternalToolForm
    {
        private Label label_Version;
        private Label label_Game;
        private Label label_Connection;
        private Label label_Save;
        private Label label_Help;

        [RequiredApi]
		private IMemoryApi? _maybeMemAPI { get; set; }

		private IMemoryApi _memAPI => _maybeMemAPI ?? throw new NullReferenceException();

        private bool isVanilla;
        private TcpStream? stream;
        private List<byte> prevSaveData = new List<byte>();
        private Save? prevSave;

        private bool gameOk = false;
        private bool connectionOk = false;
        private bool saveOk = false;

		public MainForm()
		{
			InitializeComponent();
			ClientSize = new Size(640, 480);
			Text = "OoT autotracker";
			SuspendLayout();
			ResumeLayout();
		}

		public bool AskSaveChanges() => true;

		public void Restart() {
            if (this.stream != null) this.stream.Disconnect().Dispose();
            this.stream = null;
            label_Connection.Text = "Connection: waiting for game";
            this.connectionOk = false;
            UpdateHelpLabel();
            if (this.prevSave != null) this.prevSave.Dispose();
            this.prevSave = null;
            label_Save.Text = "Save: waiting for game";
            this.saveOk = false;
            UpdateHelpLabel();
            if (GlobalWin.Game.Name == "Null")
            {
                label_Game.Text = "Not playing anything";
                this.gameOk = false;
                UpdateHelpLabel();
            }
            else
            {
                var rom_ident = _memAPI.ReadByteRange(0x20, 0x18, "ROM");
                if (!Enumerable.SequenceEqual(rom_ident.GetRange(0, 0x15), new List<byte>(Encoding.UTF8.GetBytes("THE LEGEND OF ZELDA \0"))))
                {
                    label_Game.Text = $"Game: Expected OoT or OoTR, found {GlobalWin.Game.Name} ({string.Join<byte>(", ", rom_ident.GetRange(0, 0x15))})";
                    this.gameOk = false;
                    UpdateHelpLabel();
                }
                else
                {
                    var version = rom_ident.GetRange(0x15, 3);
                    this.isVanilla = Enumerable.SequenceEqual(version, new List<byte>(new byte[] { 0, 0, 0 }));
                    if (this.isVanilla)
                    {
                        label_Game.Text = "Playing OoT (vanilla)";
                        this.gameOk = true;
                        UpdateHelpLabel();
                    }
                    else
                    {
                        label_Game.Text = $"Playing OoTR version {version[0]}.{version[1]}.{version[2]}";
                        this.gameOk = true;
                        UpdateHelpLabel();
                    }
                    using (var stream_res = new TcpStreamResult(IPAddress.IPv6Loopback))
                    {
                        if (stream_res.IsOk())
                        {
                            if (this.stream != null) this.stream.Disconnect().Dispose();
                            this.stream = new TcpStream(stream_res);
                            label_Connection.Text = "Connected";
                            this.connectionOk = true;
                            UpdateHelpLabel();
                            if (this.isVanilla)
                            {
                                using (var knowledge = new Knowledge(true))
                                {
                                    knowledge.Send(this.stream);
                                }
                            }
                        }
                        else
                        {
                            using (StringHandle err = stream_res.DebugErr())
                            {
                                label_Connection.Text = $"Failed to connect: {err.AsString()}";
                                this.connectionOk = false;
                                UpdateHelpLabel();
                            }
                        }
                    }
                }
            }
        }

		public void UpdateValues(ToolFormUpdateType type)
        {
            if (GlobalWin.Game.Name == "Null") return;
            if (type != ToolFormUpdateType.PreFrame) return; //TODO setting to also enable auto-tracking during turbo (ToolFormUpdateType.FastPreFrame)?
            var save_data = _memAPI.ReadByteRange(0x11a5d0, 0x1450, "RDRAM");
            if (save_data != prevSaveData)
            {
                prevSaveData = save_data;
                using (SaveResult state_res = new SaveResult(save_data))
                {
                    bool is_ok = state_res.IsOk();
                    if (is_ok)
                    {
                        Save save = new Save(state_res);
                        {
                            using (StringHandle debug = save.Debug())
                            {
                                label_Save.Text = $"Save data ok, last checked {DateTime.Now}";
                                this.saveOk = true;
                                UpdateHelpLabel();
                            }
                            if (prevSave == null)
                            {
                                if (this.stream != null)
                                {
                                    using (IoResult io_res = save.Send(this.stream))
                                    {
                                        if (!io_res.IsOk())
                                        {
                                            if (this.stream != null) this.stream.Dispose();
                                            this.stream = null;
                                            using (StringHandle err = io_res.DebugErr())
                                            {
                                                label_Connection.Text = $"Failed to send save data: {err.AsString()}";
                                                this.connectionOk = false;
                                                UpdateHelpLabel();
                                            }
                                        }
                                        else
                                        {
                                            label_Connection.Text = $"Connected, initial save data sent {DateTime.Now}";
                                            this.connectionOk = true;
                                            UpdateHelpLabel();
                                        }
                                    }
                                }
                                prevSave = save;
                            }
                            else if (!save.Equals(prevSave))
                            {
                                if (this.stream != null)
                                {
                                    using (SavesDiff diff = prevSave.Diff(save))
                                    {
                                        using (IoResult io_res = diff.Send(this.stream))
                                        {
                                            if (!io_res.IsOk())
                                            {
                                                if (this.stream != null) this.stream.Dispose();
                                                this.stream = null;
                                                using (StringHandle err = io_res.DebugErr())
                                                {
                                                    label_Connection.Text = $"Failed to send save data: {err.AsString()}";
                                                    this.connectionOk = false;
                                                    UpdateHelpLabel();
                                                }
                                            }
                                            else
                                            {
                                                label_Connection.Text = $"Connected, save data last sent {DateTime.Now}";
                                                this.connectionOk = false;
                                                UpdateHelpLabel();
                                            }
                                        }
                                    }
                                }
                                prevSave.Dispose();
                                prevSave = save;
                            }
                            else
                            {
                                save.Dispose();
                            }
                        }
                    }
                    else
                    {
                        using (StringHandle err = state_res.DebugErr())
                        {
                            label_Save.Text = $"Error reading save data: {err.AsString()}";
                            this.saveOk = false;
                            UpdateHelpLabel();
                        }
                    }
                }
            }
        }

        private void UpdateHelpLabel()
        {
            if (this.gameOk && this.connectionOk && this.saveOk)
            {
                label_Help.Text = "You can now minimize this window. To stop auto-tracking, close this window.";
            }
            else
            {
                label_Help.Text = "If you need help, you can ask in #setup-support on Discord.";
            }
        }

        private void InitializeComponent()
        {
            this.label_Version = new Label();
            this.label_Game = new Label();
            this.label_Connection = new Label();
            this.label_Save = new Label();
            this.label_Help = new Label();
            this.SuspendLayout();
            //
            // label_Version
            //
            this.label_Version.AutoSize = true;
            this.label_Version.Location = new Point(12, 9);
            this.label_Version.Name = "label_Version";
            this.label_Version.Size = new Size(96, 25);
            this.label_Version.TabIndex = 0;
            this.label_Version.Text = $"OoT autotracker version {Native.version_string().AsString()}";
            // 
            // label_Game
            // 
            this.label_Game.AutoSize = true;
            this.label_Game.Location = new Point(12, 34);
            this.label_Game.Name = "label_Game";
            this.label_Game.Size = new Size(96, 25);
            this.label_Game.TabIndex = 1;
            this.label_Game.Text = "Game: loading";
            // 
            // label_Connection
            // 
            this.label_Connection.AutoSize = true;
            this.label_Connection.Location = new Point(12, 59);
            this.label_Connection.Name = "label_Connection";
            this.label_Connection.Size = new Size(96, 25);
            this.label_Connection.TabIndex = 2;
            this.label_Connection.Text = "Connection: waiting for game";
            // 
            // label_Save
            // 
            this.label_Save.AutoSize = true;
            this.label_Save.Location = new Point(12, 84);
            this.label_Save.Name = "label_Save";
            this.label_Save.Size = new Size(96, 25);
            this.label_Save.TabIndex = 3;
            this.label_Save.Text = "Save: waiting for game";
            //
            // label_Help
            //
            this.label_Help.AutoSize = true;
            this.label_Help.Location = new Point(12, 109);
            this.label_Help.Name = "label_Help";
            this.label_Help.Size = new Size(96, 25);
            this.label_Help.TabIndex = 4;
            this.label_Help.Text = "If you need help, you can ask in #setup-support on Discord.";
            // 
            // MainForm
            // 
            this.ClientSize = new Size(274, 229);
            this.Controls.Add(this.label_Version);
            this.Controls.Add(this.label_Game);
            this.Controls.Add(this.label_Connection);
            this.Controls.Add(this.label_Save);
            this.Controls.Add(this.label_Help);
            this.Name = "MainForm";
            this.ResumeLayout(false);
            this.PerformLayout();
        }
    }
}
