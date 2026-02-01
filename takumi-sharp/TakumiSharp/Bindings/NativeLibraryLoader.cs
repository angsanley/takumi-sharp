using System.Reflection;
using System.Runtime.InteropServices;

namespace TakumiSharp.Bindings
{
  public static class NativeLibraryLoader
  {
    private static bool _initialized = false;

    public static void Initialize()
    {
      if (_initialized) return;

      NativeLibrary.SetDllImportResolver(typeof(NativeBindings).Assembly, DllImportResolver);
      _initialized = true;
    }

    private static IntPtr DllImportResolver(string libraryName, Assembly assembly, DllImportSearchPath? searchPath)
    {
      if (libraryName == "takumi")
      {
        string rid = RuntimeInformation.RuntimeIdentifier;
        string[] possibleNames = GetPossibleLibraryNames(libraryName);

        foreach (string libName in possibleNames)
        {
          // Try runtime-specific path first
          string libPath = Path.Combine(AppContext.BaseDirectory, "runtimes", rid, "native", libName);

          if (File.Exists(libPath) && NativeLibrary.TryLoad(libPath, out IntPtr handle))
            return handle;

          // Fallback to simplified architecture path
          string arch = RuntimeInformation.ProcessArchitecture.ToString().ToLower();
          libPath = Path.Combine(AppContext.BaseDirectory, "runtimes", $"win-{arch}", "native", libName);

          if (File.Exists(libPath) && NativeLibrary.TryLoad(libPath, out handle))
            return handle;
        }
      }

      return IntPtr.Zero;
    }

    private static string[] GetPossibleLibraryNames(string baseName)
    {
      if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
      {
        return [$"{baseName}.dll", $"lib{baseName}.dll"];
      }
      else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
      {
        return [$"lib{baseName}.so", $"{baseName}.so"];
      }
      else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
      {
        return [$"lib{baseName}.dylib", $"{baseName}.dylib"];
      }

      return [baseName];
    }
  }
}