package tools.hdhunter;

public class HDHunter {
    public static final byte MODE_REQUEST = 1;
    public static final byte MODE_RESPONSE = 2;
    public static final byte MODE_SCGI = 4;
    public static final byte MODE_FASTCGI = 8;
    public static final byte MODE_AJP = 16;
    public static final byte MODE_UWSGI = 32;

    public static native int setContentLength(long length, int mode);
    public static native int setChunkedEncoding(byte chunked, int mode);
    public static native int incConsumedLength(long length, int mode);
    public static native int incBodyLength(long length, int mode);
    public static native int markMessageProcessed(int mode);

    static {
        System.loadLibrary("hdhunter_rt_java");
    }
}
