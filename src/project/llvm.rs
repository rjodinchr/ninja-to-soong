use std::collections::HashSet;

use crate::filesystem;
use crate::soongfile::SoongFile;
use crate::soongmodule::SoongModule;
use crate::target::BuildTarget;
use crate::utils::*;

const DST_BUILD_PREFIX: &str = "cmake_generated";

pub struct LLVM<'a> {
    src_root: &'a str,
    build_root: &'a str,
}

impl<'a> LLVM<'a> {
    pub fn new(src_root: &'a str, build_root: &'a str) -> Self {
        LLVM {
            src_root,
            build_root,
        }
    }
}

impl<'a> crate::project::Project<'a> for LLVM<'a> {
    fn generate(self, targets: Vec<BuildTarget>) -> Result<String, String> {
        let mut file = SoongFile::new(self.src_root, "", self.build_root, "llvm-project_");
        if let Err(err) = file.generate(
            vec![
                "libLLVMAggressiveInstCombine.a",
                "libLLVMAnalysis.a",
                "libLLVMAsmParser.a",
                "libLLVMBinaryFormat.a",
                "libLLVMBitReader.a",
                "libLLVMBitWriter.a",
                "libLLVMBitstreamReader.a",
                "libLLVMCFGuard.a",
                "libLLVMCGData.a",
                "libLLVMCodeGenTypes.a",
                "libLLVMCodeGen.a",
                "libLLVMCore.a",
                "libLLVMCoroutines.a",
                "libLLVMCoverage.a",
                "libLLVMDebugInfoBTF.a",
                "libLLVMDebugInfoCodeView.a",
                "libLLVMDebugInfoDWARF.a",
                "libLLVMDebugInfoMSF.a",
                "libLLVMDebugInfoPDB.a",
                "libLLVMDemangle.a",
                "libLLVMExtensions.a",
                "libLLVMFrontendDriver.a",
                "libLLVMFrontendHLSL.a",
                "libLLVMFrontendOffloading.a",
                "libLLVMFrontendOpenMP.a",
                "libLLVMHipStdPar.a",
                "libLLVMIRPrinter.a",
                "libLLVMIRReader.a",
                "libLLVMInstCombine.a",
                "libLLVMInstrumentation.a",
                "libLLVMLTO.a",
                "libLLVMLinker.a",
                "libLLVMMCParser.a",
                "libLLVMMC.a",
                "libLLVMObjCARCOpts.a",
                "libLLVMObject.a",
                "libLLVMOption.a",
                "libLLVMPasses.a",
                "libLLVMProfileData.a",
                "libLLVMRemarks.a",
                "libLLVMSandboxIR.a",
                "libLLVMScalarOpts.a",
                "libLLVMSupport.a",
                "libLLVMSymbolize.a",
                "libLLVMTargetParser.a",
                "libLLVMTarget.a",
                "libLLVMTextAPI.a",
                "libLLVMTransformUtils.a",
                "libLLVMVectorize.a",
                "libLLVMWindowsDriver.a",
                "libLLVMipo.a",
                "libclangAPINotes.a",
                "libclangASTMatchers.a",
                "libclangAST.a",
                "libclangAnalysis.a",
                "libclangBasic.a",
                "libclangCodeGen.a",
                "libclangDriver.a",
                "libclangEdit.a",
                "libclangFrontend.a",
                "libclangLex.a",
                "libclangParse.a",
                "libclangSema.a",
                "libclangSerialization.a",
                "libclangSupport.a",
            ],
            targets,
            &self,
        ) {
            return Err(err);
        }
        let mut generated_headers = file.get_generated_headers();
        let include_directories = file.get_include_directories();

        let dirs_to_remove = vec![DST_BUILD_PREFIX];
        for dir_to_remove in dirs_to_remove {
            let dir = add_slash_suffix(self.src_root) + dir_to_remove;
            if touch::exists(&dir) {
                if let Err(err) = std::fs::remove_dir_all(&dir) {
                    return error!(format!("remove_dir_all failed: {err}"));
                }
            }
        }

        let missing_generated_headers = vec![
            "include/llvm/Config/llvm-config.h",
            "include/llvm/Config/abi-breaking.h",
            "include/llvm/Config/config.h",
            "include/llvm/Config/Targets.def",
            "include/llvm/Config/AsmPrinters.def",
            "include/llvm/Config/AsmParsers.def",
            "include/llvm/Config/Disassemblers.def",
            "include/llvm/Config/TargetMCAs.def",
            "include/llvm/Support/Extension.def",
            "include/llvm/Support/VCSRevision.h",
            "tools/clang/lib/Basic/VCSVersion.inc",
            "tools/clang/include/clang/Basic/Version.inc",
            "tools/clang/include/clang/Config/config.h",
            "tools/libclc/clspv--.bc",
            "tools/libclc/clspv64--.bc",
        ];
        for header in missing_generated_headers {
            generated_headers.insert(header.to_string());
        }
        match filesystem::copy_files(
            generated_headers,
            &add_slash_suffix(self.build_root),
            &(add_slash_suffix(self.src_root) + &add_slash_suffix(DST_BUILD_PREFIX)),
        ) {
            Ok(msg) => println!("{msg}"),
            Err(err) => return Err(err),
        }
        match filesystem::touch_directories(&include_directories, &add_slash_suffix(self.src_root))
        {
            Ok(msg) => println!("{msg}"),
            Err(err) => return Err(err),
        }

        if let Err(err) = file.add_module(SoongModule::new_cc_library_headers(
            LLVM_HEADERS,
            "llvm/include",
        )) {
            return Err(err);
        }
        if let Err(err) = file.add_module(SoongModule::new_cc_library_headers(
            CLANG_HEADERS,
            "clang/include",
        )) {
            return Err(err);
        }

        // for clspv
        let opencl_c_base = "clang/lib/Headers/opencl-c-base.h";
        if let Err(err) = file.add_module(SoongModule::new_copy_genrule(
            clang_headers_name("clang", opencl_c_base),
            opencl_c_base.to_string(),
            opencl_c_base.rsplit_once("/").unwrap().1.to_string(),
        )) {
            return Err(err);
        }
        let clspv_bc = DST_BUILD_PREFIX.to_string() + "/tools/libclc/clspv--.bc";
        if let Err(err) = file.add_module(SoongModule::new_copy_genrule(
            llvm_headers_name(DST_BUILD_PREFIX, &clspv_bc),
            clspv_bc.clone(),
            clspv_bc.rsplit_once("/").unwrap().1.to_string(),
        )) {
            return Err(err);
        }
        let clspv64_bc = DST_BUILD_PREFIX.to_string() + "/tools/libclc/clspv64--.bc";
        if let Err(err) = file.add_module(SoongModule::new_copy_genrule(
            llvm_headers_name(DST_BUILD_PREFIX, &clspv64_bc),
            clspv64_bc.clone(),
            clspv64_bc.rsplit_once("/").unwrap().1.to_string(),
        )) {
            return Err(err);
        }

        return file.write(self.src_root);
    }
    fn parse_custom_command_inputs(
        &self,
        _: &Vec<String>,
    ) -> Result<(HashSet<String>, HashSet<String>, HashSet<(String, String)>), String> {
        return error!(format!(
            "parse_custom_command_inputs not supported for llvm project"
        ));
    }
    fn get_default_defines(&self) -> HashSet<String> {
        [
            "-Wno-error".to_string(),
            "-Wno-unreachable-code-loop-increment".to_string(),
        ]
        .into()
    }
    fn ignore_target(&self, input: &String) -> bool {
        !input.starts_with("lib")
    }
    fn ignore_include(&self, _: &str) -> bool {
        false
    }
    fn rework_include(&self, include: &str) -> String {
        include.replace(self.build_root, DST_BUILD_PREFIX)
    }
    fn get_headers_to_copy(&self, headers: &HashSet<String>) -> HashSet<String> {
        let mut set = HashSet::new();
        for header in headers {
            set.insert(header.clone());
        }
        return set;
    }
    fn get_headers_to_generate(&self, _: &HashSet<String>) -> HashSet<String> {
        return HashSet::new();
    }
    fn get_object_header_libs(&self) -> HashSet<String> {
        return HashSet::new();
    }
}
