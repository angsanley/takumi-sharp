namespace TakumiSharp.Bindings
{
  internal static partial class NativeBindings
  {
    static NativeBindings()
    {
      NativeLibraryLoader.Initialize();
    }
  }
}