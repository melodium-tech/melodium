use crate::reference::load_core_collection;
use crate::tools::{
    check_program, describe_element, get_cicd_migration_guide, get_language_guide,
    get_program_info, list_library_elements, read_book_chapter, search_book, search_reference,
    CheckProgramRequest, DescribeElementRequest, GetCicdMigrationGuideRequest,
    GetLanguageGuideRequest, GetProgramInfoRequest, ListLibraryElementsRequest,
    ReadBookChapterRequest, SearchBookRequest, SearchReferenceRequest,
};
use melodium_common::descriptor::Collection;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Json, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ServerHandler,
};
use std::sync::Arc;

#[derive(Clone)]
#[allow(dead_code)]
pub struct MelodiumMcp {
    tool_router: ToolRouter<Self>,
    /// Every compiled-in Mélodium standard library package, loaded once in
    /// mock mode at startup and reused across reference-lookup tool calls.
    collection: Arc<Collection>,
}

impl MelodiumMcp {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            collection: load_core_collection(),
        }
    }
}

#[tool_router]
impl MelodiumMcp {
    #[tool(
        description = "Parse and validate a Mélodium program file (.mel, Compo.toml, or .jeu), returning structured errors and the list of available entrypoints. Runs entirely against mocked library implementations: no real filesystem, network, subprocess, or database I/O beyond reading the given file is performed."
    )]
    fn check_program(
        &self,
        Parameters(request): Parameters<CheckProgramRequest>,
    ) -> Json<crate::tools::CheckProgramResult> {
        Json(check_program(request))
    }

    #[tool(
        description = "Parse a Mélodium program file and describe each of its entrypoints: identifier, documentation, and parameters (name, const/var, type, default value)."
    )]
    fn get_program_info(
        &self,
        Parameters(request): Parameters<GetProgramInfoRequest>,
    ) -> Json<crate::tools::GetProgramInfoResult> {
        Json(get_program_info(request))
    }

    #[tool(
        description = "List Mélodium standard library elements (treatments, functions, models, contexts, data types), optionally filtered by area path (e.g. `std/flow`, `http`, `sql`) and/or kind."
    )]
    fn list_library_elements(
        &self,
        Parameters(request): Parameters<ListLibraryElementsRequest>,
    ) -> Json<crate::tools::ListLibraryElementsResult> {
        Json(list_library_elements(request, &self.collection))
    }

    #[tool(
        description = "Describe the full signature of one Mélodium standard library element by its identifier (e.g. `std/flow::emit`, `http/server::HttpServer`): documentation, generics, parameters, and (for treatments) inputs, outputs, required models and contexts."
    )]
    fn describe_element(
        &self,
        Parameters(request): Parameters<DescribeElementRequest>,
    ) -> Json<crate::tools::DescribeElementResult> {
        Json(describe_element(request, &self.collection))
    }

    #[tool(
        description = "Search Mélodium standard library identifiers and documentation for a keyword, returning matching elements with a short snippet."
    )]
    fn search_reference(
        &self,
        Parameters(request): Parameters<SearchReferenceRequest>,
    ) -> Json<crate::tools::SearchReferenceResult> {
        Json(search_reference(request, &self.collection))
    }

    #[tool(
        description = "Explanation of Mélodium's dataflow execution model (treatments, tracks, models, contexts, connections, generics, project layout) aimed at an AI reading or writing Mélodium code, in Markdown."
    )]
    fn get_language_guide(
        &self,
        Parameters(request): Parameters<GetLanguageGuideRequest>,
    ) -> Json<crate::tools::GetLanguageGuideResult> {
        Json(get_language_guide(request))
    }

    #[tool(
        description = "Reference guide for migrating CI/CD pipelines (GitHub Actions or GitLab CI) to Mélodium's `cicd` package: concept mapping, treatments, and data types."
    )]
    fn get_cicd_migration_guide(
        &self,
        Parameters(request): Parameters<GetCicdMigrationGuideRequest>,
    ) -> Json<crate::tools::GetCicdMigrationGuideResult> {
        Json(get_cicd_migration_guide(request))
    }

    #[tool(
        description = "Search the Mélodium book (the language's narrative reference documentation) by keyword across chapter titles and content. Omit the query to list every chapter instead."
    )]
    fn search_book(
        &self,
        Parameters(request): Parameters<SearchBookRequest>,
    ) -> Json<crate::tools::SearchBookResult> {
        Json(search_book(request))
    }

    #[tool(
        description = "Read the full Markdown content of one Mélodium book chapter, addressed by the path reported by search_book (e.g. `programming/elements/functions.md`)."
    )]
    fn read_book_chapter(
        &self,
        Parameters(request): Parameters<ReadBookChapterRequest>,
    ) -> Json<crate::tools::ReadBookChapterResult> {
        Json(read_book_chapter(request))
    }
}

#[tool_handler]
impl ServerHandler for MelodiumMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "Validate and inspect Mélodium (.mel) programs, and browse the Mélodium \
                 standard library and book reference. Use check_program to type-check a program \
                 file, get_program_info to inspect its entrypoints, list_library_elements/\
                 search_reference to browse the standard library, and describe_element for a \
                 specific element's full signature. Use get_language_guide for an explanation of \
                 the dataflow execution model, get_cicd_migration_guide when migrating a GitHub \
                 Actions or GitLab CI pipeline, and search_book/read_book_chapter to browse the \
                 Mélodium book's narrative documentation.",
        )
    }
}
